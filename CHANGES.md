# cactus 1.0.6 (2020-06-11)

* Added `cactus::ArcCactus` variant which internally uses an `Arc` instead of
  an `Rc`, allowing `ArcCactus`es to be shared across threads.


# cactus 1.0.5 (2019-11-21)

* License as dual Apache-2.0/MIT (instead of a more complex, and little
  understood, triple license of Apache-2.0/MIT/UPL-1.0).


# cactus 1.0.4 (2018-12-18)

* Ported to Rust 2018.


# cactus 1.0.3 (2018-04-04)

* Triple licence Apache-2 / MIT / UPL.


# cactus 1.0.2 (2018-02-05)

* Cactuses are now hashable.

* Remove take_or_clone_val() and replace it with try_unwrap, modelled on
  Rc::try_unwrap.

* Shortcut eq() comparison with Rc::ptr_eq, turning the best case comparison
  from O(n) to O(1) (though the worst case remains O(n)).


# cactus 1.0.1 (2018-02-01)

* Tentatively add take_or_clone_val().


# cactus 1.0.0 (2017-10-10)

First stable release.
