![Maintenance](https://img.shields.io/badge/maintenance-experimental-blue.svg)

# CYKL-RS
This is a Rust project implementing several heuristic solvers for the travelling salesman problem and its related problems. I intend to clearly separate the algorithm (solver) part and data structure part so that the data structures implemented in this library can be reused elsewhere for novel algorithms. Moreover, it also has another benefit that better data structures can be implemented without affecting the internal working of algorithms.

Currently, the library has working version of data structure with seemingly good performance. In the next step, I will focus on implementing the LKH algorithm and $\alpha$-nearness for the candidate generation.

## Benchmarks
The implementation of array and two-level list (TLL) is based on the description presented in Fredman's
paper [[3]](#3). The following operations are considered essential in the context of TSP solver:
- **```get(id)```**: returns a node for the given index ```id```. Theoretically, this function should compute in ```O(1)``` time.
- **```successor(a)```**: returns a node that directly succeeds a node ```a``` in the tour. Theoretically, this function should compute in ```O(1)``` time
- **```predecessor(a)```**: returns a node that directly preceeds a node ```a``` in the tour. Theoretically, this function should compute in ```O(1)``` time
- **```between(a, b, c)```**: checks whether ```b``` is between of ```a``` and ```c``` in a forward traversal direction. Theoretically, this function should compute in ```O(1)``` time
- **```flip(a, b, c, d)```**: breaks the edges ```(a, b)``` and ```(c, d)```, then forms new edges ```(a, c)``` and ```(b, d)```. This operation will affect not only four input nodes but also their neighbours. For array, this operation can take up to ```O(N)``` time to complete, while TLL can achieve ```O(sqrt(N))``` performance.
    - For TLL, the computation time for ```flip``` operation heavily depends on whether the operation affects only elements of a certain segment or affects entire segments or across many segments.
    - In the current benchmark, we set the maximum number of elements for each segment in TLL to ```100```.
    - Case 1 of ```flip``` benchmark involves reversing 99 elements in a segment. In this case, TLL must iterate through 99 nodes and reverse all of them. 
    - Case 2 involves reversing 100 elements, all of which lie in the same segment. Since the entire segment is affected, TLL only needs to reverse the bit flag of the corresponding segment. It is expected that case 2 is when TLL shines the most.
    - Case 3 is the most complicated. It involves reversing 900 nodes across multiple segments. In this case, TLL has to split and merge segments in order to get a layout such that case 2 can be applied.

**NOTE**:
- Since the benchmarked functions are called in isolated environment, the result shown below
shouldn't be used to form a final decision. A real benchmark will be carried out once the main algorithm is fully implemented.
- The unit for computation is *nanosecond (ns)* for all entries. The crate [criterion](https://crates.io/crates/criterion) is used to benchmark the performance of the implemented data structures.
- In the table below ```ED``` denotes function invocation through enum-dispatch. In this library, we use the crate [enum_dispatch](https://crates.io/crates/enum_dispatch) to handle the code generation for such dispatch.

| |Array |Array (ED) | TLL | TLL (ED) |
--- | --- | --- | --- | ---
|**get**| 0.793 | 0.812 | 0.481 | 0.469
|**successor**|3.367 | 3.324 | 0.938 | 1.144
|**predecessor**|0.851 | 0.797 | 0.473 | 0.477
|**between**| 0.948 | 0.942| 2.9533 | 2.989
|**flip (case 1)**|447.75| 446.26| 106.64 | 106.99
|**flip (case 2)**|458.51| 453.55| 14.303 | 15.247
|**flip (case 3)**|4267.3 | 4329.2 | 69.806 | 72.694

The benchmark shows that TLL outperforms Array in most of the cases. This is due to some of optimisations for TLL implementation and also partly due to Rust's optimisation for [NonNull](https://doc.rust-lang.org/std/ptr/struct.NonNull.html) pointer, while Array implementation is only a naive one. We will try to optimise Array in the future.

## References
<a id="1">[1]</a> S. Lin; B. W. Kernighan(1973). "An Effective Heuristic Algorithm for the Traveling-Salesman Problem". Operations Research. 21 (2): 498–516. [doi:10.1287/opre.21.2.498](https://pubsonline.informs.org/doi/abs/10.1287/opre.21.2.498).

<a id="2">[2]</a> K. Helsgaun (2000). "An effective implementation of the Lin–Kernighan traveling salesman heuristic". European Journal of Operational Research. 126 (1): 106-130. [doi:10.1016/S0377-2217(99)00284-2](https://doi.org/10.1016/S0377-2217(99)00284-2).

<a id="3">[3]</a> M. Fredman et al. (1995). "Data Structures for Traveling Salesmen". J. Algorithms 18: 432-479. [doi:10.1006/jagm.1995.1018](http://citeseer.ist.psu.edu/viewdoc/download;jsessionid=72CE6E9143B3CB461E627995CE1E419E?doi=10.1.1.49.570&rep=rep1&type=pdf).

<a id="4">[4]</a> D. Sleator and R. Tarjan (1985). "Self-Adjusting Binary Search Trees" Journal of the ACM 32 (3): 652-686. [doi:10.1145/3828.3835](https://www.cs.cmu.edu/~sleator/papers/self-adjusting.pdf).

<a id="5">[5]</a> M. Chrobak, T.Szymacha and A.Krawczyk (1990). "A data structure useful for finding Hamiltonian cycles". Theoretical Computer Science 71 (3): 419-424. [doi:10.1016/0304-3975(90)90053-K](https://www.sciencedirect.com/science/article/pii/030439759090053K).

<a id="6">[6]</a> Z. Michaelwicz and D. Fogel (2004). "How to solve it: Modern Heuristics". Springer, 2nd Edition. [Amazon.com](https://www.amazon.com/How-Solve-Heuristics-Zbigniew-Michalewicz/dp/3540224947).

<a id="7">[7]</a> D. Applegate et al. "Finding Tours in the TSP".  	Forschungsinstitut für Diskrete Mathematik Report No. 99885, Universität Bonn. [U. Waterloo](http://www.math.uwaterloo.ca/tsp/methods/papers/lk_report.html).

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.