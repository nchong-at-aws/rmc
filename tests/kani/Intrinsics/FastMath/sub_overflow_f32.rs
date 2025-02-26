// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Check that `fsub_fast` triggers overflow checks
// kani-verify-fail

#![feature(core_intrinsics)]

#[kani::proof]
fn main() {
    let x: f32 = kani::any();
    let y: f32 = kani::any();
    let _z = unsafe { std::intrinsics::fsub_fast(x, y) };
}
