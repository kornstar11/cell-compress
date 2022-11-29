# cell-compress-rs

Quick project that repersents a Grid (basicly a matrix) and tries to keep in the most efficient format. It does this by sampling the currently occupied spots, and if nessesary converting between sparse or dense repersentations.

* Why not just use a hashmap for this?
  * Vec<Vec<Cell>> Should be faster to iterate if it is filled enough, and only has small gaps.
* Why not just use Vec<Vec<>>?
  * Adding and removing elements from a Vec<> can be expensive, if the vec is large and the element was removed from the head. 
  * To get around this you can leave something in its place (in our case a None), however, it then becomes wasteful of memory. To solve this, we just convert to a HashMap, when enough space exists between elements.

Some tests have been included to show this.
