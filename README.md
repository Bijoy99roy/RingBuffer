# ringbuffer

A small, minimal, lock-free multi-producer single-consumer ring buffer written in Rust.

## Overview

This crate provides a compact, high-performance ring buffer implementation for use when one or more thread
produces items and another thread consumes them. It's designed for low-latency scenarios and benchmarking
against channels or other queue implementations.

## Features

- MPSC (multi-producer, single-consumer) lock-free ring buffer
- Fixed capacity defined at compile time via a const generic

## Build

- `cargo build --release`

## Run

- `cargo run`

## Benchmarks

Benchmarks use Criterion.

- `cargo bench`
