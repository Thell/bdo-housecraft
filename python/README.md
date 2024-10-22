# Optimal Bi-State Subset Selection Problem.

The Bi-state subset selection problem is a linear programming problem to select a subset of items that minimize the total weight while satisfying several constraints. This type of problem is a generalization of the “knapsack problem” or a “subset selection problem”.

The problem can be described as follows:

**Objective**:
  - Minimize: ∑ wᵢ xᵢ

where:
  - wᵢ is the weight of item i
  - xᵢ is a binary variable that takes the value 1 if item i is selected and 0
    otherwise.

**Subject to**:
  - xᵢ - xⱼ ≤ 0 for all (i, j) such that item j is the parent of item i
  - ∑ v₁ᵢ y₁ᵢ ≥ lb₁
  - ∑ v₂ᵢ y₂ᵢ ≥ lb₂
  - y₁ᵢ + y₂ᵢ - xᵢ = 0
  - xᵢ, y₁ᵢ, y₂ᵢ ∈ {0, 1}

where:
  - lb₁ and lb₂ are lower bounds on the sum of the values of the selected items
    in each state
  - v₁ᵢ and v₂ᵢ are the values of item i in states 1 and 2, respectively
  - y₁ᵢ and y₂ᵢ are binary variables indicating whether item i is selected in 
    state 1 or state 2, respectively, and 0 otherwise


The first set of constraints ensures that every item selected requires all ancestor items to be selected. The second and third sets of constraints ensure that the sum of the values of the selected items in each state is greater than or equal to a lower bound. The fourth set of constraints ensures that each item can be in at most one state and if selected it must be in at least one state. The last set of constraints specifies that all variables are binary.


## Notes on ideal counts up to Mountain of Winter release.

Town counts:
Key  |   Town               |  Ideals              |    w/warehouse & workers |
-----|----------------------|----------------------|------------------------- |
5    |   Velia              | 186,624              |    1,166,400 |
32   |   Heidel             | 28,227,424,942,080   |    12,282,316,378,675,200 |
52   |   Glish              | 14                   |    78 |
77   |   Calpheon City      | (1.63e21 in python)  |    (2.53e22 in python) |
88   |   Olvia              | 823,680              |    40,365,000 |
107  |   Keplan             | 8,064                |    674,100 |
120  |   Port Epheria       | 124,362              |    40,777,724 |
126  |   Trent              | 273                  |    16,100 |
182  |   Iliya Island       | 240                  |    560 |
202  |   Altinova           | 31,933,440,000       |    4,534,185,600,000 |
221  |   Tarif              | 53,460               |    396,900 |
229  |   Valencia City      | (1.94e21 in python)  |    (1.18e22 in python) |
601  |   Shakatu            | 36                   |    525 |
605  |   Sand Grain Bazaar  | 144                  |    1,815 |
619  |   Ancado Inner Harbor| 12                   |    33 |
693  |   Arehaza            | 9                    |    32 |
694  |   Muiquun            | 4                    |    4 |
706  |   Old Wisdom Tree    | 4                    |    4 |
735  |   Grána              | 524,288              |    1,769,472 |
873  |   Duvencrune         | 131,072              |    663,552 |
955  |   O'draxxia          | 6                    |    126 |
1124 |   Eilton             | 24                   |    324 |
