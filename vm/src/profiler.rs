/*
Performance Profiler for VelaVM

This module provides performance profiling capabilities for the Vela Virtual Machine,
including instruction counting, execution time measurement, and memory usage tracking.
*/

use crate::bytecode::Instruction;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Performance profiler
pub struct Profiler {
    instruction_counts: HashMap<String, u64>,
    execution_times: HashMap<String, Duration>,
    memory_usage: Vec<usize>,
    start_time: Option<Instant>,
    current_instruction: Option<String>,
}

impl Profiler {
    /// Create a new profiler
    pub fn new() -> Self {
        Self {
            instruction_counts: HashMap::new(),
            execution_times: HashMap::new(),
            memory_usage: Vec::new(),
            start_time: None,
            current_instruction: None,
        }
    }

    /// Start profiling
    pub fn start(&mut self) {
        self.start_time = Some(Instant::now());
    }

    /// Stop profiling
    pub fn stop(&mut self) {
        self.start_time = None;
        self.current_instruction = None;
    }

    /// Record execution of an instruction
    pub fn record_instruction(&mut self, instruction: &Instruction) {
        let instr_name = format!("{:?}", instruction);

        // Count instruction
        *self.instruction_counts.entry(instr_name.clone()).or_insert(0) += 1;

        // Start timing this instruction
        self.current_instruction = Some(instr_name);
        self.start_time = Some(Instant::now());
    }

    /// Record completion of current instruction
    pub fn finish_instruction(&mut self) {
        if let (Some(start), Some(instr)) = (self.start_time.take(), self.current_instruction.take()) {
            let duration = start.elapsed();
            *self.execution_times.entry(instr).or_insert(Duration::ZERO) += duration;
        }
    }

    /// Record memory usage
    pub fn record_memory(&mut self, usage: usize) {
        self.memory_usage.push(usage);
    }

    /// Get instruction execution counts
    pub fn get_instruction_counts(&self) -> &HashMap<String, u64> {
        &self.instruction_counts
    }

    /// Get total execution time per instruction type
    pub fn get_execution_times(&self) -> &HashMap<String, Duration> {
        &self.execution_times
    }

    /// Get memory usage history
    pub fn get_memory_usage(&self) -> &[usize] {
        &self.memory_usage
    }

    /// Get total execution time
    pub fn get_total_time(&self) -> Duration {
        self.execution_times.values().sum()
    }

    /// Get most executed instruction
    pub fn get_most_executed_instruction(&self) -> Option<(&String, u64)> {
        self.instruction_counts
            .iter()
            .max_by_key(|(_, &count)| count)
            .map(|(instr, count)| (instr, *count))
    }

    /// Get slowest instruction type
    pub fn get_slowest_instruction(&self) -> Option<(&String, Duration)> {
        self.execution_times
            .iter()
            .max_by_key(|(_, &duration)| duration)
            .map(|(instr, duration)| (instr, *duration))
    }

    /// Generate profiling report
    pub fn generate_report(&self) -> String {
        let mut report = String::from("=== VelaVM Performance Profile ===\n\n");

        report.push_str("Instruction Counts:\n");
        for (instr, count) in &self.instruction_counts {
            report.push_str(&format!("  {}: {}\n", instr, count));
        }

        report.push_str("\nExecution Times:\n");
        for (instr, duration) in &self.execution_times {
            report.push_str(&format!("  {}: {:.2}ms\n", instr, duration.as_millis()));
        }

        let total_time = self.get_total_time();
        report.push_str(&format!("\nTotal Execution Time: {:.2}ms\n", total_time.as_millis()));

        if let Some((instr, count)) = self.get_most_executed_instruction() {
            report.push_str(&format!("Most Executed: {} ({} times)\n", instr, count));
        }

        if let Some((instr, duration)) = self.get_slowest_instruction() {
            report.push_str(&format!("Slowest: {} ({:.2}ms)\n", instr, duration.as_millis()));
        }

        if !self.memory_usage.is_empty() {
            let avg_memory = self.memory_usage.iter().sum::<usize>() / self.memory_usage.len();
            let max_memory = *self.memory_usage.iter().max().unwrap_or(&0);
            report.push_str(&format!("Average Memory Usage: {} bytes\n", avg_memory));
            report.push_str(&format!("Peak Memory Usage: {} bytes\n", max_memory));
        }

        report
    }

    /// Reset profiler state
    pub fn reset(&mut self) {
        self.instruction_counts.clear();
        self.execution_times.clear();
        self.memory_usage.clear();
        self.start_time = None;
        self.current_instruction = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profiler_creation() {
        let profiler = Profiler::new();
        assert!(profiler.instruction_counts.is_empty());
        assert!(profiler.execution_times.is_empty());
        assert!(profiler.memory_usage.is_empty());
    }

    #[test]
    fn test_instruction_counting() {
        let mut profiler = Profiler::new();

        profiler.record_instruction(&Instruction::Add);
        profiler.record_instruction(&Instruction::Add);
        profiler.record_instruction(&Instruction::Sub);

        let counts = profiler.get_instruction_counts();
        assert_eq!(counts.get("Add"), Some(&2));
        assert_eq!(counts.get("Sub"), Some(&1));
    }

    #[test]
    fn test_memory_recording() {
        let mut profiler = Profiler::new();

        profiler.record_memory(1024);
        profiler.record_memory(2048);
        profiler.record_memory(1536);

        let memory = profiler.get_memory_usage();
        assert_eq!(memory, &[1024, 2048, 1536]);
    }

    #[test]
    fn test_report_generation() {
        let mut profiler = Profiler::new();

        profiler.record_instruction(&Instruction::Add);
        profiler.record_memory(1024);

        let report = profiler.generate_report();
        assert!(report.contains("VelaVM Performance Profile"));
        assert!(report.contains("Add: 1"));
    }
}