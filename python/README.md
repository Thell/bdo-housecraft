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
