//! # Timeout
//! This module contains some internal utilities for handling async oneshot channels
//! that have a response timeout.

use std::{collections::{HashSet, VecDeque}, sync::{Mutex, MutexGuard}, time::{Duration, Instant}};

use slotmap::{new_key_type, SlotMap};
use tokio::sync::oneshot::{self, Receiver, Sender};


/// # [`TimeoutChannels`]
/// This struct wraps a [`SlotMap`] of oneshot channels and a queue.
/// As new channels are created, they are inserted into the [`SlotMap`]
/// and their IDs pushed to the end of the queue. Channels will be dropped
/// if either they expire, or if they are retrieved. The [`TimeoutChannels::tick`]
/// method should be called in a loop, either in a separate task or
/// selected upon in a main loop.
///
/// # Generics
/// The `D` generic is the data type that is sent over the channels.
pub struct TimeoutChannels<C: slotmap::Key, D> {
    /// The duration after which channels will be removed.
    timeout: Duration,
    /// The [`SlotMap`] of channel senders.
    /// This is wrapped with a [`Mutex`]
    /// to allow the struct to be called
    /// from threaded code.
    channels: Mutex<SlotMap<C, Sender<D>>>,
    /// A deque of channel keys paired with the 
    /// time of their insertion. Elements are always
    /// inserted at the end, and under a lock,
    /// which guarentees that their insertion time 
    /// is in ascending order, and thus also their
    /// expiration time.
    /// This is wrapped in a mutex, as it needs
    /// to be accessed across threads in a synchronized manner.
    timeout_queue: Mutex<VecDeque<(Instant, C)>>,
}

impl<C: slotmap::Key, D> TimeoutChannels<C, D> {
    /// # [`TimeoutChannels::new`]
    /// Creates a new, empty [`TimeoutChannels`] instance with the given
    /// timeout duration.
    pub fn new(timeout: Duration) -> Self {
        Self {
            timeout,
            channels: Mutex::new(SlotMap::with_key()),
            timeout_queue: Mutex::new(VecDeque::new()),
        }
    }

    /// # [`TimeoutChannels::recover`]
    /// Recovers from a poisoned mutex. This will force affected channels to close.
    fn recover(&self, tq: Option<MutexGuard<'_, VecDeque<(Instant, C)>>>, channels: Option<MutexGuard<'_, SlotMap<C, Sender<D>>>>) {
        
        // Get the timeout queue, regardless of poison status
        let mut tq = tq.unwrap_or_else(|| match self.timeout_queue.lock() {
            Ok(tq) => tq,
            Err(e) => e.into_inner()
        });

        // Same with the channels
        let mut channels = channels.unwrap_or_else(|| match self.channels.lock() {
            Ok(tq) => tq,
            Err(e) => e.into_inner()
        });

        // Build a set of channels in the timeout queue
        let tq_set = tq.iter().map(|i| i.1).collect::<HashSet<_>>();
    
        // Do the same with the channels
        let channels_set = channels.keys().collect::<HashSet<_>>();

        // Get the symmetric difference
        let difference = tq_set.symmetric_difference(&channels_set).collect::<HashSet<_>>();

        // Check if the timeout queue has duplicate keys.
        let tq_duplicated = tq_set.len() != tq.len();

        // Check if the channels set has duplicate keys. This is unrecoverable
        let channels_duplicated = channels_set.len() != channels.len();

        // If the sets are equal, and there are no duplicate keys,
        // depoison the mutexes and return
        if difference.is_empty() && !tq_duplicated && !channels_duplicated {
            self.timeout_queue.clear_poison();
            self.channels.clear_poison();
            return;
        }

        // Iterate over every element in the difference to resolve the issue
        for i in difference {

            // If it is in tq_set, then it is not in the channel.
            // This responder can't be recovered, so we just remove it.
            if tq_set.contains(i) {
                channels.remove(*i);
            }

            // If it is in channel_set, then it is not in the timeout queue.
            // This can't be recovered either, as that means
            // the receiver was not sent to the user.
            if channels_set.contains(i) {
                tq.retain(|v| v.1 != *i);
            }
        }


        // If the timeout queue has duplicates, fix it.
        if tq_duplicated {
            // Create a set to contain the seen elements.
            let mut seen = HashSet::<C>::new();

            // All indexes to remove.
            let mut remove_indexes = Vec::<usize>::new();

            // Iterate in reverse to prioritize recently created channels.
            // If there are duplicates, the timeout must have been missed.
            for (i, v) in tq.iter().rev().enumerate() {
                // If this id has been seen, designate for removal
                if seen.contains(&v.1) {
                    remove_indexes.push(i)
                }

                // Mark as seen
                seen.insert(v.1);
            }

            // Remove all designated indexes
            for i in remove_indexes {
                tq.remove(i);
            }
        }

        // The slotmap shouldn't have duplicate items, so we can depoison and return.
        self.timeout_queue.clear_poison();
        self.channels.clear_poison();
    }


    /// # [`TimeoutChannels::add`]
    /// Adds a new channel, returning it's recever and key.
    /// 
    pub fn add(&self) -> (Receiver<D>, C) {

        // Lock the timeout queue mutex.
        // This is done first to ensure that the ordering
        // of the elements is kept.
        let mut timeout_queue = self.timeout_queue.lock()
            .unwrap_or_else(|e| {
            // Recover the mutex if it was poisoned
            self.recover(Some(e.into_inner()), None);
            self.timeout_queue.lock().expect("Mutex should no longer be poisoned")
        });

        // Create the channel
        let (sender, receiver) = oneshot::channel::<D>();

        // Now lock the channels mutex, insert the sender,
        // and immediately drop the lock.
        let mut channels = self.channels.lock()
            .unwrap_or_else(|e| {
            // Recover the mutex if it was poisoned
            self.recover(None, Some(e.into_inner()));
            self.channels.lock().expect("Mutex should no longer be poisoned")
        });

        let key = channels.insert(sender);

        drop(channels);

        // Note: At this point, there must be no errors until the value is inserted
        // into the timeout_queue, or the channel will never be removed.

        // Create the instant for the insertion time
        let insertion_time = Instant::now();

        // Insert into the timeout_queue
        timeout_queue.push_back((insertion_time, key));

        drop(timeout_queue);

        // Return both the key and receiver
        (receiver, key)
    }

    /// # [`TimeoutChannels::pop`]
    /// Pops a sender based on the ID, returning it and 
    /// removing it from the slotmap and timeout queue.
    /// Returns [`None`] if the channel doesn't exist
    /// 
    pub fn pop(&self, key: C) -> Option<Sender<D>> {

        // Pop it from the slotmap
        let channel = self.channels.lock()
            .unwrap_or_else(|e| {
                // Recover the mutex if it was poisoned
                self.recover(None, Some(e.into_inner()));
                self.channels.lock().expect("Mutex should no longer be poisoned")
            })
            .remove(key)?;

        // Remove from the queue
        let mut timeout_queue = self.timeout_queue.lock()
            .unwrap_or_else(|e| {
                // Recover the mutex if it was poisoned
                self.recover(Some(e.into_inner()), None);
                self.timeout_queue.lock().expect("Mutex should no longer be poisoned")
            });

        // We only need to remove one, because we will only ever insert one
        // unique key. This allows us to remove in the loop, and exit early.
        for (i, v) in timeout_queue.iter().enumerate() {
            if v.1 == key {
                timeout_queue.remove(i);
                break;
            }
        }

        // Return the channel
        Some(channel)
    }

    /// # [`TimeoutChannels::tick`]
    /// This is the body of the main loop that checks for channels
    /// to drop. This can be a long running function, so it should
    /// be either selected upon or run in a separate task.
    /// 
    pub async fn tick(&self) {
        
        // Get the first element from the list
        let Some(first) = self.timeout_queue.lock()
            .unwrap_or_else(|e| {
                // Recover the mutex if it was poisoned
                self.recover(Some(e.into_inner()), None);
                self.timeout_queue.lock().expect("Mutex should no longer be poisoned")
            }).front().copied() else {
            return;
        };

        // Wait until the instant is elapsed. The lock will not be held across
        // the await point, as it is dropped in the previous statement.
        tokio::time::sleep_until((first.0 + self.timeout).into()).await;

        // Remove elements from both the timeout queue and slotmap
        // until one is reached that isn't timed out.
        let mut timeout_queue = self.timeout_queue.lock()
            .unwrap_or_else(|e| {
                // Recover the mutex if it was poisoned
                self.recover(Some(e.into_inner()), None);
                self.timeout_queue.lock().expect("Mutex should no longer be poisoned")
            });
        let mut channels = self.channels.lock()
            .unwrap_or_else(|e| {
                // Recover the mutex if it was poisoned
                self.recover(None, Some(e.into_inner()));
                self.channels.lock().expect("Mutex should no longer be poisoned")
            });

        loop {
            // Return early if the list is exhausted
            let Some(item) = timeout_queue.front().copied() else {
                return;
            };

            // If the instant is not elapsed, quit
            if item.0.elapsed() < self.timeout {
                break;
            }

            // Remove the element if the timeout is elapsed
            timeout_queue.pop_front();
            channels.remove(item.1); // This should cause an error on the receiving end.
        }
    }

}