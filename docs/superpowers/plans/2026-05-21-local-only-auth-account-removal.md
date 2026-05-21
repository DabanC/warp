# Local-only Auth and Account Removal Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Remove Warp login, registration, auth state, remote anonymous user creation, and account prompts while preserving local terminal startup.

**Architecture:** Replace remote user requirements with a local installation identity and remove auth UI entry points before pruning auth dependencies. Each task compiles before moving to the next dependency layer.

**Tech Stack:** Rust, Cargo, Warp app modules, local-only verification scripts.

---

This plan is intentionally separate from the foundation plan. It starts after `script/local-only/verify --self-test` passes and the foundation patch stack has been exported.
