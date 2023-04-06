# Optimal Bi-State Subset Selection Problem.

The Bi-state subset selection problem is a linear programming problem to select a subset of items that minimize the total weight while satisfying several constraints. This type of problem is a generalization of the “knapsack problem” or a “subset selection problem”.

The problem can be described as follows:

**Objective**:
  - Minimize: ∑ᵢ wᵢ xᵢ

where wᵢ is the weight of item i and xᵢ is a binary variable that takes the value 1 if item i is selected and 0 otherwise.

**Subject to**:
  - xⱼ ≤ xᵢ for all (i, j) such that item j is a descendant of item i
  - ∑ᵢ v₁ᵢ y₁ᵢ ≥ lb₁
  - ∑ᵢ v₂ᵢ y₂ᵢ ≥ lb₂
  - y₁ᵢ + y₂ᵢ ≤ 1 for all i
  - xᵢ ≤ y₁ᵢ + y₂ᵢ for all i
  - y₁ᵢ * v₁ᵢ ≥ y₁ᵢ for all i
  - y₂ᵢ * v₂ᵢ ≥ y₂ᵢ for all i
  - xᵢ, y₁ᵢ, y₂ᵢ ∈ {0, 1} for all i

where v₁ᵢ and v₂ᵢ are the values of item i in states 1 and 2, respectively; y₁ᵢ and y₂ᵢ are binary variables that take the value 1 if item i is in state 1 or 2, respectively, and 0 otherwise; and lb₁ and lb₂ are lower bounds on the sum of the values of the selected items in each state.

The first set of constraints ensures that every item selected requires all ancestor items to be selected. The second and third sets of constraints ensure that the sum of the values of the selected items in each state is greater than or equal to a lower bound. The fourth set of constraints ensures that each item can be in at most one state. The fifth set of constraints ensures that if an item is selected, it must be assigned to one of the two states. The sixth and seventh sets of constraints ensure that if an item is in state 1 or state 2, its associated value must be greater than zero. The last set of constraints specifies that all variables are binary.
