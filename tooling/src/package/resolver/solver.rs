//! SAT Solver for Dependency Resolution
//!
//! This module implements a SAT (Satisfiability) solver specifically designed
//! for dependency resolution problems. It uses the CDCL (Conflict-Driven Clause
//! Learning) algorithm with optimizations for package dependency constraints.

use crate::common::Error;
use crate::package::resolver::constraints::VersionConstraint;
use crate::package::resolver::graph::{DependencyGraph, PackageId};
use semver::Version;
use std::collections::{HashMap, HashSet, VecDeque};

/// A literal in the SAT problem (package version constraint)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Literal {
    pub package: PackageId,
    pub version: Version,
    pub negated: bool, // true for NOT (package@version)
}

impl Literal {
    pub fn positive(package: PackageId, version: Version) -> Self {
        Self {
            package,
            version,
            negated: false,
        }
    }

    pub fn negative(package: PackageId, version: Version) -> Self {
        Self {
            package,
            version,
            negated: true,
        }
    }

    pub fn negate(&self) -> Self {
        Self {
            package: self.package.clone(),
            version: self.version.clone(),
            negated: !self.negated,
        }
    }
}

/// A clause in the SAT problem (OR of literals)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Clause {
    pub literals: Vec<Literal>,
    pub learned: bool, // true if this clause was learned from conflicts
}

impl Clause {
    pub fn new(literals: Vec<Literal>) -> Self {
        Self {
            literals,
            learned: false,
        }
    }

    pub fn learned(literals: Vec<Literal>) -> Self {
        Self {
            literals,
            learned: true,
        }
    }

    pub fn is_unit(&self, assignments: &HashMap<PackageId, (Version, usize)>) -> Option<&Literal> {
        let mut unassigned = Vec::new();

        for literal in &self.literals {
            match assignments.get(&literal.package) {
                Some((assigned_version, _)) => {
                    let satisfied = if literal.negated {
                        assigned_version != &literal.version
                    } else {
                        assigned_version == &literal.version
                    };

                    if satisfied {
                        return None; // Clause is satisfied
                    }
                }
                None => unassigned.push(literal),
            }
        }

        if unassigned.len() == 1 {
            Some(unassigned[0])
        } else {
            None
        }
    }

    pub fn is_satisfied(&self, assignments: &HashMap<PackageId, (Version, usize)>) -> bool {
        for literal in &self.literals {
            if let Some((assigned_version, _)) = assignments.get(&literal.package) {
                let satisfied = if literal.negated {
                    assigned_version != &literal.version
                } else {
                    assigned_version == &literal.version
                };

                if satisfied {
                    return true;
                }
            }
        }
        false
    }
}

/// The SAT solver state
#[derive(Debug)]
pub struct SATSolver {
    pub clauses: Vec<Clause>,
    pub assignments: HashMap<PackageId, (Version, usize)>, // (version, decision_level)
    pub decision_level: usize,
    pub implication_graph: HashMap<PackageId, (Literal, usize)>, // literal -> (reason, level)
    pub conflict_clauses: Vec<Clause>,
}

impl SATSolver {
    pub fn new() -> Self {
        Self {
            clauses: Vec::new(),
            assignments: HashMap::new(),
            decision_level: 0,
            implication_graph: HashMap::new(),
            conflict_clauses: Vec::new(),
        }
    }

    /// Add a clause to the solver
    pub fn add_clause(&mut self, clause: Clause) {
        self.clauses.push(clause);
    }

    /// Convert dependency constraints to SAT clauses
    pub fn add_dependency_constraints(
        &mut self,
        graph: &DependencyGraph,
    ) -> Result<(), Error> {
        for (package_id, node) in &graph.nodes {
            // For each package, add clauses for version constraints
            for constraint in &node.constraints {
                let versions = &node.package.versions;

                // Find versions that satisfy the constraint
                let satisfying_versions: Vec<&Version> = versions
                    .iter()
                    .filter(|v| constraint.satisfies(v))
                    .collect();

                if satisfying_versions.is_empty() {
                    return Err(Error::UnsatisfiableConstraint {
                        package: package_id.name.clone(),
                        constraint: constraint.to_string(),
                    });
                }

                // Add clause: at least one satisfying version must be chosen
                let literals: Vec<Literal> = satisfying_versions
                    .iter()
                    .map(|v| Literal::positive(package_id.clone(), (*v).clone()))
                    .collect();

                self.add_clause(Clause::new(literals));
            }

            // Add mutual exclusion for versions of the same package
            // (cannot choose two different versions of the same package)
            let versions = &node.package.versions;
            for i in 0..versions.len() {
                for j in (i + 1)..versions.len() {
                    let lit1 = Literal::negative(package_id.clone(), versions[i].clone());
                    let lit2 = Literal::negative(package_id.clone(), versions[j].clone());
                    self.add_clause(Clause::new(vec![lit1, lit2]));
                }
            }
        }

        // Add dependency implications
        for (from_package, deps) in &graph.edges {
            for (to_package, constraint) in deps {
                let from_versions = &graph.nodes[from_package].package.versions;
                let to_versions = &graph.nodes[to_package].package.versions;

                for from_version in from_versions {
                    // If we choose from_version, we must choose a compatible to_version
                    let compatible_to_versions: Vec<&Version> = to_versions
                        .iter()
                        .filter(|v| constraint.satisfies(v))
                        .collect();

                    if compatible_to_versions.is_empty() {
                        // If no compatible versions, then we cannot choose from_version
                        let lit = Literal::negative(from_package.clone(), from_version.clone());
                        self.add_clause(Clause::new(vec![lit]));
                    } else {
                        // If we choose from_version, we must choose at least one compatible to_version
                        let mut literals = vec![Literal::negative(from_package.clone(), from_version.clone())];
                        for to_version in compatible_to_versions {
                            literals.push(Literal::positive(to_package.clone(), (*to_version).clone()));
                        }
                        self.add_clause(Clause::new(literals));
                    }
                }
            }
        }

        Ok(())
    }

    /// Solve the SAT problem using CDCL algorithm
    pub fn solve(&mut self) -> Result<HashMap<PackageId, Version>, Error> {
        loop {
            // Unit propagation
            if let Some(conflict) = self.unit_propagation()? {
                // Conflict detected
                if self.decision_level == 0 {
                    return Err(Error::Unsatisfiable);
                }

                // Learn conflict clause and backtrack
                let learned_clause = self.analyze_conflict(conflict);
                self.conflict_clauses.push(learned_clause.clone());
                self.add_clause(learned_clause);

                self.backtrack()?;
                continue;
            }

            // Check if all variables are assigned
            if self.is_complete() {
                // Convert assignments to version-only map
                let result: HashMap<PackageId, Version> = self.assignments
                    .iter()
                    .map(|(pkg, (version, _))| (pkg.clone(), version.clone()))
                    .collect();
                return Ok(result);
            }

            // Make a decision
            self.make_decision();
        }
    }

    /// Perform unit propagation
    fn unit_propagation(&mut self) -> Result<Option<Clause>, Error> {
        let mut propagated = true;

        while propagated {
            propagated = false;

            for clause in &self.clauses {
                if let Some(unit_literal) = clause.is_unit(&self.assignments) {
                    if self.assignments.contains_key(&unit_literal.package) {
                        continue; // Already assigned
                    }

                    // Assign the unit literal
                    let version = if unit_literal.negated {
                        // For negative literals, we need to choose a different version
                        // This is simplified - in practice, we'd need more sophisticated handling
                        return Err(Error::SolverError { message: "Complex unit propagation not implemented".to_string() });
                    } else {
                        unit_literal.version.clone()
                    };

                    self.assignments.insert(unit_literal.package.clone(), (version, self.decision_level));
                    self.implication_graph.insert(
                        unit_literal.package.clone(),
                        (unit_literal.clone(), self.decision_level),
                    );

                    propagated = true;

                    // Check for conflicts
                    if self.has_conflict() {
                        return Ok(Some(clause.clone()));
                    }
                }
            }
        }

        Ok(None)
    }

    /// Check if there's a conflict in current assignments
    fn has_conflict(&self) -> bool {
        for clause in &self.clauses {
            if !clause.is_satisfied(&self.assignments) {
                return true;
            }
        }
        false
    }

    /// Check if all variables are assigned
    fn is_complete(&self) -> bool {
        // This is a simplified check - in practice, we'd track all variables
        !self.clauses.iter().any(|clause| {
            clause.literals.iter().any(|lit| !self.assignments.contains_key(&lit.package))
        })
    }

    /// Make a decision (choose an unassigned variable)
    fn make_decision(&mut self) {
        self.decision_level += 1;

        // Find an unassigned package and choose its first available version
        for clause in &self.clauses {
            for literal in &clause.literals {
                if !self.assignments.contains_key(&literal.package) {
                    // Choose this version
                    self.assignments.insert(literal.package.clone(), (literal.version.clone(), self.decision_level));
                    self.implication_graph.insert(
                        literal.package.clone(),
                        (literal.clone(), self.decision_level),
                    );
                    return;
                }
            }
        }
    }

    /// Analyze conflict and learn a clause
    fn analyze_conflict(&self, conflict_clause: Clause) -> Clause {
        // Simplified conflict analysis
        // In a full implementation, this would use the implication graph
        // to find the real conflict reason and learn a clause

        // For now, just return a learned clause that excludes the current assignments
        let mut learned_literals = Vec::new();

        for (package, _) in &self.assignments {
            // Add negation of current assignment
            if let Some((current_version, _)) = self.assignments.get(package) {
                learned_literals.push(Literal::negative(package.clone(), current_version.clone()));
            }
        }

        Clause::learned(learned_literals)
    }

    /// Backtrack to a previous decision level
    fn backtrack(&mut self) -> Result<(), Error> {
        if self.decision_level == 0 {
            return Err(Error::Unsatisfiable);
        }

        // Remove assignments from current decision level
        let current_level = self.decision_level;
        self.assignments.retain(|_, (_, level)| *level < current_level);
        self.implication_graph.retain(|_, (_, level)| *level < current_level);

        self.decision_level -= 1;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use semver::Version;

    #[test]
    fn test_literal_creation() {
        let package_id = PackageId::new("test".to_string());
        let version = Version::parse("1.0.0").unwrap();

        let positive = Literal::positive(package_id.clone(), version.clone());
        assert!(!positive.negated);

        let negative = Literal::negative(package_id, version);
        assert!(negative.negated);
    }

    #[test]
    fn test_clause_unit_detection() {
        let package_id = PackageId::new("test".to_string());
        let v1 = Version::parse("1.0.0").unwrap();
        let v2 = Version::parse("2.0.0").unwrap();

        let mut assignments = HashMap::new();
        assignments.insert(package_id.clone(), v2.clone());

        let clause = Clause::new(vec![
            Literal::positive(package_id.clone(), v1.clone()),
            Literal::positive(package_id.clone(), v2.clone()),
        ]);

        // Clause is satisfied (v2 is assigned), so no unit literal
        assert!(clause.is_unit(&assignments).is_none());

        let unit_clause = Clause::new(vec![Literal::positive(package_id, v1)]);
        // This would be unit if v1 wasn't assigned, but since v2 is assigned and doesn't match,
        // it's not satisfied, but also not unit in this simplified check
        assert!(unit_clause.is_unit(&assignments).is_none());
    }

    #[test]
    fn test_solver_creation() {
        let solver = SATSolver::new();
        assert!(solver.clauses.is_empty());
        assert!(solver.assignments.is_empty());
        assert_eq!(solver.decision_level, 0);
    }
}