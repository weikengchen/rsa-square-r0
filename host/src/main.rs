use core::str::FromStr;
use methods::{METHOD_ELF, METHOD_ID};
use num_bigint::{BigUint, RandBigInt};
use num_traits::identities::Zero;
use rand_chacha::rand_core::SeedableRng;
use rand_chacha::ChaCha20Rng;
use risc0_zkvm::{default_prover, ExecutorEnv};

pub struct Task {
    // 22 limbs, each of length 96 bits = 3 x u32 = 12 x u8
    pub a: Vec<u8>,
    // 43 limbs, each of length 224 bits = 7 x u32 = 28 x u8
    // total: 1204 bytes
    pub long_form_c: Vec<u8>,
    // 22 limbs, each of length 96 bits = 3 x u32 = 12 x u8
    // total: 264 bytes
    pub k: Vec<u8>,
    // 43 limbs, each of length 224 bits = 7 x u32 = 28 x u8
    // total: 1204 bytes
    pub long_form_kn: Vec<u8>,
}

fn main() {
    let mut prng = ChaCha20Rng::seed_from_u64(0u64);

    let a = prng.gen_biguint(2048);
    let aa = &a * &a;
    let n = BigUint::from_str(
        "22181287481343866536926164726351287326530456851865740940302258624292918842046294265777588938243700158420966504059481663514441470940350196901315671547076005234970874435909476092497483551273288093189364709035514616037071211153823131905024178182878201024915500433097297265826798822817484748700216324125712309789054401424099125210527384783630725436400275931057214172116786047287671841780210364049070913138670556222022084829676330760494242212963241225957072902927387309610872757297833214507573774777580968710434530894604337230857277368168283766335313014325255932691808839056156851505239358105335763858378332776753927248103"
    ).unwrap();
    let k = &aa / &n;
    println!("res should be {}", &aa % &n);

    let mut a_bytes = [0u8; 264];
    a_bytes[0..a.to_bytes_le().len()].copy_from_slice(&a.to_bytes_le());

    let mut k_bytes = [0u8; 264];
    k_bytes[0..k.to_bytes_le().len()].copy_from_slice(&k.to_bytes_le());

    let mut n_bytes = [0u8; 264];
    n_bytes[0..n.to_bytes_le().len()].copy_from_slice(&n.to_bytes_le());

    let mut a_limbs = vec![];
    for i in 0..22 {
        a_limbs.push(BigUint::from_bytes_le(&a_bytes[i * 12..i * 12 + 12]));
    }

    let mut c_limbs = vec![];
    for _ in 0..43 {
        c_limbs.push(BigUint::zero());
    }
    for i in 0..22 {
        for j in 0..22 {
            c_limbs[i + j] += &a_limbs[i] * &a_limbs[j];
        }
    }

    let mut k_limbs = vec![];
    for i in 0..22 {
        k_limbs.push(BigUint::from_bytes_le(&k_bytes[i * 12..i * 12 + 12]));
    }

    let mut n_limbs = vec![];
    for i in 0..22 {
        n_limbs.push(BigUint::from_bytes_le(&n_bytes[i * 12..i * 12 + 12]));
    }

    let mut kn_limbs = vec![];
    for _ in 0..43 {
        kn_limbs.push(BigUint::zero());
    }
    for i in 0..22 {
        for j in 0..22 {
            kn_limbs[i + j] += &k_limbs[i] * &n_limbs[j];
        }
    }

    let mut c_bytes = Vec::new();
    for i in 0..43 {
        let mut bytes = [0u8; 28];
        bytes[0..c_limbs[i].to_bytes_le().len()].copy_from_slice(&c_limbs[i].to_bytes_le());
        c_bytes.extend_from_slice(&bytes);
    }

    let mut kn_bytes = Vec::new();
    for i in 0..43 {
        let mut bytes = [0u8; 28];
        bytes[0..kn_limbs[i].to_bytes_le().len()].copy_from_slice(&kn_limbs[i].to_bytes_le());
        kn_bytes.extend_from_slice(&bytes);
    }

    let task = Task {
        a: a_bytes.to_vec(),
        long_form_c: c_bytes,
        k: k_bytes.to_vec(),
        long_form_kn: kn_bytes,
    };

    let env = ExecutorEnv::builder()
        .write_slice(&task.a)
        .write_slice(&task.long_form_c)
        .write_slice(&task.k)
        .write_slice(&task.long_form_kn)
        .build()
        .unwrap();

    let prover = default_prover();

    let timer = std::time::Instant::now();
    let receipt = prover.prove_elf(env, METHOD_ELF).unwrap();
    println!("time: {}", timer.elapsed().as_secs_f64());
    receipt.verify(METHOD_ID).unwrap();
}
