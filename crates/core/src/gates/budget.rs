// Copyright 2024-2025 Aprio One AB, Sweden
// Author: Kenneth Pernyer, kenneth@aprio.one
// SPDX-License-Identifier: LicenseRef-Proprietary
// All rights reserved. This source code is proprietary and confidential.
// Unauthorized copying, modification, or distribution is strictly prohibited.

//! Budget types for guaranteed termination.
//!
//! Budget newtypes provide checked arithmetic to prevent:
//! - Underflow (budget going negative)
//! - Silent exhaustion (missing the moment budget hits zero)
//!
//! Each budget type has a `tick()` method that returns `Option<StopReason>`
//! when exhausted, ensuring the engine always knows when to stop.

use serde::{Deserialize, Serialize};

use super::stop::StopReason;

/// Cycle budget - tracks remaining execution cycles.
///
/// Each cycle is one iteration of the convergence loop.
/// When exhausted, returns `StopReason::CycleBudgetExhausted`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CycleBudget {
    remaining: u32,
    initial: u32,
}

impl CycleBudget {
    /// Create a new cycle budget with the given maximum.
    pub fn new(max: u32) -> Self {
        Self {
            remaining: max,
            initial: max,
        }
    }

    /// Get the remaining budget.
    pub fn remaining(&self) -> u32 {
        self.remaining
    }

    /// Get the initial budget.
    pub fn initial(&self) -> u32 {
        self.initial
    }

    /// Get how many cycles have been consumed.
    pub fn consumed(&self) -> u32 {
        self.initial.saturating_sub(self.remaining)
    }

    /// Check if the budget is exhausted.
    pub fn is_exhausted(&self) -> bool {
        self.remaining == 0
    }

    /// Decrement budget by one cycle.
    ///
    /// Returns `Some(StopReason::CycleBudgetExhausted)` if this tick
    /// exhausted the budget, `None` if budget remains.
    ///
    /// If already exhausted, returns the stop reason without decrementing.
    pub fn tick(&mut self) -> Option<StopReason> {
        if self.remaining == 0 {
            Some(StopReason::cycle_budget_exhausted(
                self.consumed(),
                self.initial,
            ))
        } else {
            self.remaining -= 1;
            if self.remaining == 0 {
                Some(StopReason::cycle_budget_exhausted(
                    self.consumed(),
                    self.initial,
                ))
            } else {
                None
            }
        }
    }

    /// Try to reserve multiple cycles.
    ///
    /// Returns `Ok(())` if enough budget remains, or `Err(StopReason)` if not.
    pub fn try_reserve(&mut self, cycles: u32) -> Result<(), StopReason> {
        if self.remaining >= cycles {
            self.remaining -= cycles;
            Ok(())
        } else {
            Err(StopReason::cycle_budget_exhausted(
                self.consumed(),
                self.initial,
            ))
        }
    }
}

/// Fact budget - tracks maximum facts allowed in context.
///
/// Prevents unbounded context growth.
/// When exhausted, returns `StopReason::FactBudgetExhausted`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct FactBudget {
    remaining: u32,
    initial: u32,
}

impl FactBudget {
    /// Create a new fact budget with the given maximum.
    pub fn new(max: u32) -> Self {
        Self {
            remaining: max,
            initial: max,
        }
    }

    /// Get the remaining budget.
    pub fn remaining(&self) -> u32 {
        self.remaining
    }

    /// Get the initial budget.
    pub fn initial(&self) -> u32 {
        self.initial
    }

    /// Get how many facts have been added.
    pub fn consumed(&self) -> u32 {
        self.initial.saturating_sub(self.remaining)
    }

    /// Check if the budget is exhausted.
    pub fn is_exhausted(&self) -> bool {
        self.remaining == 0
    }

    /// Decrement budget by one fact.
    ///
    /// Returns `Some(StopReason::FactBudgetExhausted)` if this addition
    /// exhausted the budget, `None` if budget remains.
    pub fn tick(&mut self) -> Option<StopReason> {
        if self.remaining == 0 {
            Some(StopReason::fact_budget_exhausted(
                self.consumed(),
                self.initial,
            ))
        } else {
            self.remaining -= 1;
            if self.remaining == 0 {
                Some(StopReason::fact_budget_exhausted(
                    self.consumed(),
                    self.initial,
                ))
            } else {
                None
            }
        }
    }

    /// Try to reserve space for multiple facts.
    pub fn try_reserve(&mut self, facts: u32) -> Result<(), StopReason> {
        if self.remaining >= facts {
            self.remaining -= facts;
            Ok(())
        } else {
            Err(StopReason::fact_budget_exhausted(
                self.consumed(),
                self.initial,
            ))
        }
    }
}

/// Token budget - tracks LLM tokens consumed.
///
/// Cost control for LLM-based operations.
/// When exhausted, returns `StopReason::TokenBudgetExhausted`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenBudget {
    remaining: u64,
    initial: u64,
}

impl TokenBudget {
    /// Create a new token budget with the given maximum.
    pub fn new(max: u64) -> Self {
        Self {
            remaining: max,
            initial: max,
        }
    }

    /// Get the remaining budget.
    pub fn remaining(&self) -> u64 {
        self.remaining
    }

    /// Get the initial budget.
    pub fn initial(&self) -> u64 {
        self.initial
    }

    /// Get how many tokens have been consumed.
    pub fn consumed(&self) -> u64 {
        self.initial.saturating_sub(self.remaining)
    }

    /// Check if the budget is exhausted.
    pub fn is_exhausted(&self) -> bool {
        self.remaining == 0
    }

    /// Consume tokens from the budget.
    ///
    /// Returns `Some(StopReason::TokenBudgetExhausted)` if this consumption
    /// exhausted the budget, `None` if budget remains.
    pub fn consume(&mut self, tokens: u64) -> Option<StopReason> {
        if self.remaining == 0 {
            return Some(StopReason::token_budget_exhausted(
                self.consumed(),
                self.initial,
            ));
        }

        if tokens >= self.remaining {
            self.remaining = 0;
            Some(StopReason::token_budget_exhausted(
                self.consumed(),
                self.initial,
            ))
        } else {
            self.remaining -= tokens;
            None
        }
    }

    /// Try to reserve tokens.
    pub fn try_reserve(&mut self, tokens: u64) -> Result<(), StopReason> {
        if self.remaining >= tokens {
            self.remaining -= tokens;
            Ok(())
        } else {
            Err(StopReason::token_budget_exhausted(
                self.consumed(),
                self.initial,
            ))
        }
    }
}

/// Combined execution budget.
///
/// Tracks all budget dimensions and checks any for exhaustion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionBudget {
    /// Cycle budget (required)
    pub cycles: CycleBudget,
    /// Fact budget (required)
    pub facts: FactBudget,
    /// Token budget (optional - only for LLM operations)
    pub tokens: Option<TokenBudget>,
}

impl ExecutionBudget {
    /// Create a new execution budget.
    pub fn new(max_cycles: u32, max_facts: u32) -> Self {
        Self {
            cycles: CycleBudget::new(max_cycles),
            facts: FactBudget::new(max_facts),
            tokens: None,
        }
    }

    /// Add a token budget.
    pub fn with_tokens(mut self, max_tokens: u64) -> Self {
        self.tokens = Some(TokenBudget::new(max_tokens));
        self
    }

    /// Check if any budget is exhausted.
    ///
    /// Returns the first exhaustion reason found.
    pub fn check_exhaustion(&self) -> Option<StopReason> {
        if self.cycles.is_exhausted() {
            return Some(StopReason::cycle_budget_exhausted(
                self.cycles.consumed(),
                self.cycles.initial(),
            ));
        }
        if self.facts.is_exhausted() {
            return Some(StopReason::fact_budget_exhausted(
                self.facts.consumed(),
                self.facts.initial(),
            ));
        }
        if let Some(ref tokens) = self.tokens {
            if tokens.is_exhausted() {
                return Some(StopReason::token_budget_exhausted(
                    tokens.consumed(),
                    tokens.initial(),
                ));
            }
        }
        None
    }

    /// Tick the cycle budget.
    pub fn tick_cycle(&mut self) -> Option<StopReason> {
        self.cycles.tick()
    }

    /// Tick the fact budget.
    pub fn tick_fact(&mut self) -> Option<StopReason> {
        self.facts.tick()
    }

    /// Consume tokens from the budget.
    pub fn consume_tokens(&mut self, tokens: u64) -> Option<StopReason> {
        self.tokens.as_mut().and_then(|t| t.consume(tokens))
    }
}

impl Default for ExecutionBudget {
    fn default() -> Self {
        Self::new(100, 10_000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // CycleBudget tests
    // ========================================================================

    #[test]
    fn cycle_budget_new() {
        let budget = CycleBudget::new(10);
        assert_eq!(budget.remaining(), 10);
        assert_eq!(budget.initial(), 10);
        assert_eq!(budget.consumed(), 0);
        assert!(!budget.is_exhausted());
    }

    #[test]
    fn cycle_budget_tick_decrements() {
        let mut budget = CycleBudget::new(3);
        assert!(budget.tick().is_none());
        assert_eq!(budget.remaining(), 2);
        assert_eq!(budget.consumed(), 1);
    }

    #[test]
    fn cycle_budget_tick_exhaustion() {
        let mut budget = CycleBudget::new(2);
        assert!(budget.tick().is_none()); // 2 -> 1
        let stop = budget.tick(); // 1 -> 0
        assert!(stop.is_some());
        if let Some(StopReason::CycleBudgetExhausted {
            cycles_executed,
            limit,
        }) = stop
        {
            assert_eq!(cycles_executed, 2);
            assert_eq!(limit, 2);
        } else {
            panic!("Expected CycleBudgetExhausted");
        }
    }

    #[test]
    fn cycle_budget_tick_already_exhausted() {
        let mut budget = CycleBudget::new(1);
        let _ = budget.tick(); // 1 -> 0
        assert!(budget.is_exhausted());
        let stop = budget.tick(); // should still return stop reason
        assert!(stop.is_some());
    }

    #[test]
    fn cycle_budget_tick_zero_initial() {
        let mut budget = CycleBudget::new(0);
        assert!(budget.is_exhausted());
        let stop = budget.tick();
        assert!(stop.is_some());
    }

    #[test]
    fn cycle_budget_try_reserve_success() {
        let mut budget = CycleBudget::new(10);
        assert!(budget.try_reserve(5).is_ok());
        assert_eq!(budget.remaining(), 5);
    }

    #[test]
    fn cycle_budget_try_reserve_insufficient() {
        let mut budget = CycleBudget::new(5);
        let result = budget.try_reserve(10);
        assert!(result.is_err());
    }

    // ========================================================================
    // FactBudget tests
    // ========================================================================

    #[test]
    fn fact_budget_new() {
        let budget = FactBudget::new(1000);
        assert_eq!(budget.remaining(), 1000);
        assert_eq!(budget.initial(), 1000);
        assert!(!budget.is_exhausted());
    }

    #[test]
    fn fact_budget_tick_exhaustion() {
        let mut budget = FactBudget::new(1);
        let stop = budget.tick();
        assert!(stop.is_some());
        assert!(budget.is_exhausted());
    }

    #[test]
    fn fact_budget_try_reserve() {
        let mut budget = FactBudget::new(100);
        assert!(budget.try_reserve(50).is_ok());
        assert_eq!(budget.remaining(), 50);
        assert!(budget.try_reserve(60).is_err());
    }

    // ========================================================================
    // TokenBudget tests
    // ========================================================================

    #[test]
    fn token_budget_new() {
        let budget = TokenBudget::new(1_000_000);
        assert_eq!(budget.remaining(), 1_000_000);
        assert_eq!(budget.initial(), 1_000_000);
        assert!(!budget.is_exhausted());
    }

    #[test]
    fn token_budget_consume() {
        let mut budget = TokenBudget::new(1000);
        assert!(budget.consume(500).is_none());
        assert_eq!(budget.remaining(), 500);
        assert_eq!(budget.consumed(), 500);
    }

    #[test]
    fn token_budget_consume_exact_exhaustion() {
        let mut budget = TokenBudget::new(100);
        let stop = budget.consume(100);
        assert!(stop.is_some());
        assert!(budget.is_exhausted());
    }

    #[test]
    fn token_budget_consume_over_exhaustion() {
        let mut budget = TokenBudget::new(50);
        let stop = budget.consume(100);
        assert!(stop.is_some());
        assert!(budget.is_exhausted());
    }

    #[test]
    fn token_budget_consume_already_exhausted() {
        let mut budget = TokenBudget::new(10);
        let _ = budget.consume(10);
        assert!(budget.is_exhausted());
        let stop = budget.consume(1);
        assert!(stop.is_some());
    }

    #[test]
    fn token_budget_try_reserve() {
        let mut budget = TokenBudget::new(1000);
        assert!(budget.try_reserve(500).is_ok());
        assert!(budget.try_reserve(600).is_err());
    }

    // ========================================================================
    // ExecutionBudget tests
    // ========================================================================

    #[test]
    fn execution_budget_new() {
        let budget = ExecutionBudget::new(100, 10_000);
        assert_eq!(budget.cycles.remaining(), 100);
        assert_eq!(budget.facts.remaining(), 10_000);
        assert!(budget.tokens.is_none());
    }

    #[test]
    fn execution_budget_with_tokens() {
        let budget = ExecutionBudget::new(100, 10_000).with_tokens(1_000_000);
        assert!(budget.tokens.is_some());
        assert_eq!(budget.tokens.unwrap().remaining(), 1_000_000);
    }

    #[test]
    fn execution_budget_default() {
        let budget = ExecutionBudget::default();
        assert_eq!(budget.cycles.remaining(), 100);
        assert_eq!(budget.facts.remaining(), 10_000);
        assert!(budget.tokens.is_none());
    }

    #[test]
    fn execution_budget_check_exhaustion_none() {
        let budget = ExecutionBudget::new(10, 100);
        assert!(budget.check_exhaustion().is_none());
    }

    #[test]
    fn execution_budget_check_exhaustion_cycles() {
        let mut budget = ExecutionBudget::new(1, 100);
        let _ = budget.tick_cycle();
        let stop = budget.check_exhaustion();
        assert!(stop.is_some());
        assert!(matches!(
            stop,
            Some(StopReason::CycleBudgetExhausted { .. })
        ));
    }

    #[test]
    fn execution_budget_check_exhaustion_facts() {
        let mut budget = ExecutionBudget::new(100, 1);
        let _ = budget.tick_fact();
        let stop = budget.check_exhaustion();
        assert!(stop.is_some());
        assert!(matches!(stop, Some(StopReason::FactBudgetExhausted { .. })));
    }

    #[test]
    fn execution_budget_check_exhaustion_tokens() {
        let mut budget = ExecutionBudget::new(100, 100).with_tokens(100);
        let _ = budget.consume_tokens(100);
        let stop = budget.check_exhaustion();
        assert!(stop.is_some());
        assert!(matches!(
            stop,
            Some(StopReason::TokenBudgetExhausted { .. })
        ));
    }

    #[test]
    fn execution_budget_tick_cycle() {
        let mut budget = ExecutionBudget::new(2, 100);
        assert!(budget.tick_cycle().is_none());
        assert!(budget.tick_cycle().is_some());
    }

    #[test]
    fn execution_budget_tick_fact() {
        let mut budget = ExecutionBudget::new(100, 2);
        assert!(budget.tick_fact().is_none());
        assert!(budget.tick_fact().is_some());
    }

    #[test]
    fn execution_budget_consume_tokens_no_budget() {
        let mut budget = ExecutionBudget::new(100, 100);
        // No token budget set
        assert!(budget.consume_tokens(1000).is_none());
    }

    #[test]
    fn execution_budget_serde_roundtrip() {
        let budget = ExecutionBudget::new(50, 500).with_tokens(5000);
        let json = serde_json::to_string(&budget).expect("serialize");
        let back: ExecutionBudget = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.cycles.remaining(), 50);
        assert_eq!(back.facts.remaining(), 500);
        assert_eq!(back.tokens.unwrap().remaining(), 5000);
    }

    // ========================================================================
    // Edge case tests
    // ========================================================================

    #[test]
    fn budget_one_tick_exhausts() {
        let mut cycle = CycleBudget::new(1);
        let stop = cycle.tick();
        assert!(stop.is_some());
        assert!(cycle.is_exhausted());

        let mut fact = FactBudget::new(1);
        let stop = fact.tick();
        assert!(stop.is_some());
        assert!(fact.is_exhausted());
    }

    #[test]
    fn budget_zero_is_immediately_exhausted() {
        let cycle = CycleBudget::new(0);
        assert!(cycle.is_exhausted());

        let fact = FactBudget::new(0);
        assert!(fact.is_exhausted());

        let token = TokenBudget::new(0);
        assert!(token.is_exhausted());
    }

    #[test]
    fn try_reserve_exact_amount() {
        let mut cycle = CycleBudget::new(5);
        assert!(cycle.try_reserve(5).is_ok());
        assert!(cycle.is_exhausted());

        let mut fact = FactBudget::new(5);
        assert!(fact.try_reserve(5).is_ok());
        assert!(fact.is_exhausted());

        let mut token = TokenBudget::new(5);
        assert!(token.try_reserve(5).is_ok());
        assert!(token.is_exhausted());
    }
}
