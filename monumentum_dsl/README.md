# monumentum_dsl

A type-safe, Rust-native query builder for Monumentum.

This crate provides a fluent API to construct and execute queries against Monumentum storage engines without using SQL strings. Queries are written as Rust code, leveraging closures and iterators, which gives compile-time type checking and eliminates SQL injection risks.