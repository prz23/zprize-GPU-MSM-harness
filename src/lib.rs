// Copyright Supranational LLC
// Licensed under the Apache License, Version 2.0, see LICENSE for details.
// SPDX-License-Identifier: Apache-2.0

pub use std::os::raw::c_void;
pub use ark_bls12_377::{Fr, G1Affine,g1::Parameters};
pub use ark_ec::AffineCurve;
pub use ark_ff::{PrimeField,bytes::ToBytes};
pub use ark_std::Zero;
pub use ark_ff::biginteger::BigInteger256;
pub use ark_ec::short_weierstrass_jacobian::GroupProjective;
pub use ark_ec::ProjectiveCurve;
pub use ark_serialize::CanonicalSerialize;
#[allow(unused_imports)]
use blst::*;

sppark::cuda_error!();

pub mod util;

#[repr(C)]
pub struct MultiScalarMultContext {
    context: *mut c_void,
}

#[cfg_attr(feature = "quiet", allow(improper_ctypes))]
extern "C" {
    fn mult_pippenger_init(
        context: *mut MultiScalarMultContext,
        points_with_infinity: *const G1Affine,
        npoints: usize,
        ffi_affine_sz: usize,
    ) -> cuda::Error;
    
    fn mult_pippenger_inf(
        context: *mut MultiScalarMultContext,
        out: *mut u64,
        points_with_infinity: *const G1Affine,
        npoints: usize,
        batch_size: usize,
        scalars: *const Fr,
        ffi_affine_sz: usize,
    ) -> cuda::Error;
}

pub fn multi_scalar_mult_init<G: AffineCurve>(
    points: &[G],
) -> MultiScalarMultContext {
    let mut ret = MultiScalarMultContext {
        context: std::ptr::null_mut(),
    };
        
    let err = unsafe {
        mult_pippenger_init(
            &mut ret,
            points as *const _ as *const G1Affine,
            points.len(),
            std::mem::size_of::<G1Affine>(),
        )
    };
    if err.code != 0 {
        panic!("{}", String::from(err));
    }

    ret
}
    
pub fn multi_scalar_mult<G: AffineCurve>(
    context: &mut MultiScalarMultContext,
    points: &[G],
    scalars: &[<G::ScalarField as PrimeField>::BigInt],
) -> Vec<G::Projective> {
    let npoints = points.len();
    if scalars.len() % npoints != 0 {
        panic!("length mismatch")
    }

    //let mut context = multi_scalar_mult_init(points);

    let batch_size = scalars.len() / npoints;
    let mut ret = vec![G::Projective::zero(); batch_size];
    let err = unsafe {
        let result_ptr = 
            &mut *(&mut ret as *mut Vec<G::Projective>
                   as *mut Vec<u64>);

        mult_pippenger_inf(
            context,
            result_ptr.as_mut_ptr(),
            points as *const _ as *const G1Affine,
            npoints, batch_size,
            scalars as *const _ as *const Fr,
            std::mem::size_of::<G1Affine>(),
        )
    };
    if err.code != 0 {
        panic!("{}", String::from(err));
    }

    ret
}
