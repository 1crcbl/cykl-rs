![Maintenance](https://img.shields.io/badge/maintenance-experimental-blue.svg)

# CYKL-RS

```cykl-rs``` is a Rust project implementing Lin-Kerninghan heuristics (LKH) algorithm for solving the travelling salesman problem and other related problems.

At the time of writing, two data structures, ```array``` and ```two-level list```, are fully implemented with all operations [[3]](#3), that are essential for building any heuristic algorithm:
- **```get(id)```**: returns a node for the given index ```id```
- **```successor(a)```**: returns a node that directly succeeds a node ```a``` in the tour.
- **```predecessor(a)```**: returns a node that directly preceeds a node ```a``` in the tour.
- **```between(a, b, c)```**: checks whether ```b``` is between of ```a``` and ```c``` in a forward traversal direction.
- **```flip(a, b, c, d)```**: this is an operation used to rearrange the sequence of nodes in a tour. It breaks the edges ```(a, b)``` and ```(c, d)```, then forms new edges ```(a, c)``` and ```(b, d)```. This operation affects not only the four input nodes but also their neighbouring nodes.

On the algorithmic side, the library offers several k-opt methods for tour manipulation. They are ```move_2_opt``` (equivalent to the ```flip``` operation mentioned above), ```move_3_opt``` and ```move_4_opt```. The method ```move_5_opt``` will be implemented soon.

For generating candidates and initial tours, the library can only offer at the moment the nearest-neighbour method. Other methods, especially ```alpha-nearness```, will be implemented soon.

All metric functions to calculate edge weights between nodes are implemented in [tspf](https://crates.io/crates/tspf), which is a parser for TSPLIB format.

## Benchmarks
The benchmark for two data structures is listed below. The unit for computation in all entries is nanosecond (ns).

| |Array | TLL |
--- | --- | --- |
|**get**| 0.716 | 0.697 | 0.700
|**successor**|3.233 | 1.155 | 1.135
|**predecessor**|0.757 | 0.708 | 0.704
|**between**| 0.218 | 3.12 | 3.122
|**flip (case 1)** <br> (flip 99 nodes in one segment)|228.04| 107.07 | 108.00
|**flip (case 2)** <br> (flip 100 nodes across two segments)|255.66| 13.648 | 14.050
|**flip (case 3)** <br> (flip 900 nodes across multiple segments)|2152.2 | 69.441 | 69.891

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

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
