===============================================
速 UUID - ``su-uuid``
===============================================


.. contents:: Table of Contents


What is ``速`` ?
========================================

``速`` means ``fast`` in Chinese.



What is this ?
========================================

``su-uuid`` is an ``uuid`` CPython Extension implemented in Rust,
which is compatable with the CPython ``uuid`` module.

By compatability,
I mean this module is using the CPython test cases for ``uuid`` module.

You can consider this as a drop-in replacement.



Why are you making this ?
========================================

As a personal practice for using Rust, PyO3 to write CPython extension.
And the reason to choose ``uuid`` module is that this module is quite simple,
but also contains mutiple things we will need to handle.
The simple part is that the module does not contains a lot of stuff,
just few functions and a class.

The good part for practicing is that it has:

* a new class and multiple methods, getter
* multiple functions
* module level constant object
* default arguments, positional arguments, keyword arguments
* good test cases in CPython
* Rust has a ``uuid`` crate



When to use this ?
========================================

As this module can pass the CPython test cases for ``uuid`` module,
this is already usable for any use case that you want to use ``uuid`` .



Build
========================================

.. code-block:: sh

    make wheel



Install
========================================

.. code-block:: sh

    make install



Testing
========================================

.. code-block:: sh

    make test



Benchmark
========================================

.. code-block:: sh

    make bench


There are some micro benchmark in the ``benchmark/`` folder.
They are written with ``pytest-benchmark``,
you can see the benchmark result by running ``make bench`` or the ``pytest benchmark/``.

Here is the rough speedup compare to the pure Python implementation in CPython:

::

    ----------------------------------------------------------------------------------------- benchmark 'uuid1': 2 tests -----------------------------------------------------------------------------------------
    Name (time in ns)            Min                     Max                  Mean                StdDev                Median                 IQR            Outliers  OPS (Kops/s)            Rounds  Iterations
    --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
    test_uuid1_rs           596.0001 (1.0)       18,153.9999 (1.0)        697.1010 (1.0)        277.6190 (1.0)        692.0000 (1.0)       29.9997 (1.0)         6;567    1,434.5123 (1.0)        7521           1
    test_uuid1_py         8,145.9993 (13.67)    328,017.9999 (18.07)    9,133.8569 (13.10)    7,114.5530 (25.63)    8,656.0003 (12.51)    439.7500 (14.66)      70;953      109.4828 (0.08)      12211           1
    --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------

    ----------------------------------------------------------------------------------------- benchmark 'uuid3': 2 tests ----------------------------------------------------------------------------------------
    Name (time in ns)            Min                     Max                  Mean                StdDev                Median                IQR            Outliers  OPS (Kops/s)            Rounds  Iterations
    -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
    test_uuid3_rs           679.0006 (1.0)      348,720.9997 (9.67)       811.5521 (1.0)      1,684.4731 (1.57)       776.0000 (1.0)      28.0006 (1.0)      123;7777    1,232.2067 (1.0)      168436           1
    test_uuid3_py         4,395.9999 (6.47)      36,069.0001 (1.0)      4,775.2736 (5.88)     1,072.4803 (1.0)      4,606.0004 (5.94)     99.0003 (3.54)    1013;1962      209.4121 (0.17)      27263           1
    -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------

    --------------------------------------------------------------------------------------- benchmark 'uuid4': 2 tests ---------------------------------------------------------------------------------------
    Name (time in ns)            Min                    Max                  Mean              StdDev                Median                IQR            Outliers  OPS (Kops/s)            Rounds  Iterations
    ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
    test_uuid4_rs           246.0001 (1.0)      15,007.0000 (1.0)        294.9147 (1.0)      350.7388 (1.0)        281.0002 (1.0)       8.0008 (1.0)      171;5594    3,390.8115 (1.0)       21173           1
    test_uuid4_py         2,748.0000 (11.17)    26,522.9992 (1.77)     2,997.0379 (10.16)    617.0421 (1.76)     2,936.0008 (10.45)    68.0011 (8.50)    1188;4438      333.6628 (0.10)      66437           1
    ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------

    --------------------------------------------------------------------------------------- benchmark 'uuid5': 2 tests ---------------------------------------------------------------------------------------
    Name (time in ns)            Min                    Max                  Mean              StdDev                Median                IQR            Outliers  OPS (Kops/s)            Rounds  Iterations
    ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
    test_uuid5_rs           664.9998 (1.0)      24,268.9994 (1.0)        768.2647 (1.0)      262.3117 (1.0)        752.9998 (1.0)      26.0006 (1.0)     1690;6876    1,301.6347 (1.0)      176274           1
    test_uuid5_py         4,345.0000 (6.53)     46,773.0006 (1.93)     4,654.5543 (6.06)     784.7173 (2.99)     4,577.0003 (6.08)     93.9990 (3.62)     479;2298      214.8433 (0.17)      38049           1



License
========================================

Apache 2.0
