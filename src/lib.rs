#![doc(
    html_logo_url = "https://www.rust-lang.org/logos/rust-logo-128x128-blk.png",
    html_favicon_url = "https://www.rust-lang.org/favicon.ico",
    html_root_url = "https://docs.rs/rand/0.5.4"
)]

#[macro_use] extern crate itertools;

#[macro_use] extern crate crunchy;

extern crate serde;
extern crate serde_json;

#[macro_use] extern crate serde_derive;

pub mod encryption;
pub mod field;
pub mod groth16;

#[doc(hidden)] pub use groth16::circuit::dummy_rep::DummyRep;
#[doc(hidden)] pub use groth16::circuit::{ASTParser, TryParse};
#[doc(hidden)] pub use groth16::circuit::{Circuit, CircuitInstance, WireId};
#[doc(hidden)] pub use groth16::coefficient_poly::CoefficientPoly;
#[doc(hidden)] pub use groth16::fr::FrLocal;
#[doc(hidden)] pub use groth16::{Proof, SigmaG1, SigmaG2, QAP};

#[cfg(test)]
mod tests {
    
    use super::field::to_field_bits;
    use super::field::z251::Z251;
    use super::groth16::Random;
    use super::*;
    use groth16::circuit::{flatten_word8, Word8};
    use groth16::fr::{G1Local, G2Local};

    extern crate tiny_keccak;
    use self::tiny_keccak::keccak256;

    #[test]
    fn simple_circuit_test() {
        // x = 4ab + c + 6
        let code = &*::std::fs::read_to_string("test_programs/simple.zk").unwrap();
        let qap: QAP<CoefficientPoly<FrLocal>> = ASTParser::try_parse(code).unwrap().into();

        // The assignments are the inputs to the circuit in the order they
        // appear in the file
        let assignments = &[
            3.into(), // a
            2.into(), // b
            4.into(), // c
        ];
        let weights = groth16::weights(code, assignments).unwrap();

        let (sigmag1, sigmag2) = groth16::setup(&qap);

        let proof = groth16::prove(&qap, (&sigmag1, &sigmag2), &weights);

        assert!(groth16::verify::<CoefficientPoly<FrLocal>, _, _, _, _>(
            (sigmag1, sigmag2),
            &vec![FrLocal::from(2), FrLocal::from(34)],
            proof
        ));

        let (sigmag1, sigmag2) = groth16::setup(&qap);

        let proof = groth16::prove(&qap, (&sigmag1, &sigmag2), &weights);

        assert!(!groth16::verify::<CoefficientPoly<FrLocal>, _, _, _, _>(
            (sigmag1, sigmag2),
            &vec![FrLocal::from(2), FrLocal::from(25)],
            proof
        ));
    }

    fn to_bits(mut num: u8) -> [u8; 8] {
        let mut bits: [u8; 8] = [0; 8];

        for i in 0..8 {
            bits[i] = num % 2;
            num = num >> 1;
        }

        bits
    }

    #[test]
    fn comparator_8bit_test() {
        // Circuit for checking if a > b
        let code = &*::std::fs::read_to_string("test_programs/8bit_comparator.zk").unwrap();
        let qap: QAP<CoefficientPoly<Z251>> = ASTParser::try_parse(code).unwrap().into();

        for _ in 0..1000 {
            let (a, b) = (Z251::random_elem(), Z251::random_elem());
            let (abits, bbits) = (to_bits(a.inner), to_bits(b.inner));

            let assignments = abits
                .iter()
                .chain(bbits.iter())
                .map(|&bit| Z251::from(bit as usize))
                .collect::<Vec<_>>();
            let weights = groth16::weights(code, &assignments).unwrap();

            let (sigmag1, sigmag2) = groth16::setup(&qap);

            let proof = groth16::prove(&qap, (&sigmag1, &sigmag2), &weights);

            if a.inner > b.inner {
                let mut inputs = vec![Z251::from(1)];
                inputs.append(
                    &mut bbits
                        .iter()
                        .map(|&bit| Z251::from(bit as usize))
                        .collect::<Vec<_>>(),
                );

                assert!(groth16::verify::<CoefficientPoly<FrLocal>, _, _, _, _>(
                    (sigmag1, sigmag2),
                    &inputs,
                    proof
                ));
            } else {
                let mut inputs = vec![Z251::from(0)];
                inputs.append(
                    &mut bbits
                        .iter()
                        .map(|&bit| Z251::from(bit as usize))
                        .collect::<Vec<_>>(),
                );

                assert!(groth16::verify::<CoefficientPoly<FrLocal>, _, _, _, _>(
                    (sigmag1, sigmag2),
                    &inputs,
                    proof
                ));
            }
        }
    }

    #[test]
    fn circuit_builder_test() {
        // Build the circuit
        let mut circuit = Circuit::<FrLocal>::new();
        let x = circuit.new_wire();
        let x_checker = circuit.new_bit_checker(x);
        let y = circuit.new_wire();
        let y_checker = circuit.new_bit_checker(y);
        let or = circuit.new_or(x, y);
        let mut instance =
            CircuitInstance::new(circuit, vec![x_checker, y_checker, or], vec![x, y], |w| {
                FrLocal::from(w.inner_id() + 1)
            });

        let qap: QAP<CoefficientPoly<FrLocal>> = QAP::from(DummyRep::from(&instance));
        let assignments = vec![FrLocal::from(0), FrLocal::from(1)];
        let weights = instance.weights(assignments);

        let (sigmag1, sigmag2) = groth16::setup(&qap);
        let proof = groth16::prove(&qap, (&sigmag1, &sigmag2), &weights);

        assert!(groth16::verify::<CoefficientPoly<FrLocal>, _, _, _, _>(
            (sigmag1, sigmag2),
            &[FrLocal::from(0), FrLocal::from(0), FrLocal::from(1)],
            proof
        ));
    }

    #[ignore]
    #[test]
    fn circuit_keccak256_single() {
        const LEN: usize = 20;
        let keccak_input: [u8; LEN] = [63; LEN];

        let tiny_keccak_output: [u8; 32] = keccak256(&keccak_input);

        let mut circuit = Circuit::<FrLocal>::new();
        let circuit_input: Vec<Word8> = circuit.new_word8_vec(LEN);
        let hash: [Word8; 32] = circuit.keccak256_stream(&circuit_input);

        let mut bit_check: Vec<WireId> = circuit.bit_check(flatten_word8(&circuit_input));
        let mut verify_wires = flatten_word8(&hash);
        verify_wires.append(&mut bit_check);

        let mut instance =
            CircuitInstance::new(circuit, verify_wires, flatten_word8(&circuit_input), |w| {
                FrLocal::from(w.inner_id() + 1)
            });

        let qap: QAP<CoefficientPoly<FrLocal>> = QAP::from(DummyRep::from(&instance));
        let assignments = to_field_bits(&keccak_input);
        let weights = instance.weights(assignments);

        let (sigmag1, sigmag2) = groth16::setup(&qap);
        let proof = groth16::prove(&qap, (&sigmag1, &sigmag2), &weights);

        let mut bit_check_vals: Vec<FrLocal> = to_field_bits(&[0; LEN]);
        let mut correct_output_vals = to_field_bits(&tiny_keccak_output);
        correct_output_vals.append(&mut bit_check_vals);

        assert!(groth16::verify::<CoefficientPoly<FrLocal>, _, _, _, _>(
            (sigmag1, sigmag2),
            &correct_output_vals,
            proof
        ));
    }
}
