use crate::{base64_url, AccountError, Secp256r1Signature};
use soroban_sdk::{crypto::Hash, panic_with_error, BytesN, Env};

#[derive(serde::Deserialize)]
struct ClientDataJson<'a> {
    challenge: &'a str,
}

/// Verify a WebAuthn secp256r1 signature against the given payload.
///
/// Steps:
/// 1. Concatenate `authenticator_data || SHA-256(client_data_json)`
/// 2. Verify the secp256r1 signature over `SHA-256(concatenated_data)`
/// 3. Parse `client_data_json` to extract the base64url-encoded `challenge`
/// 4. Encode `signature_payload` as base64url
/// 5. Verify the challenge matches the expected base64url encoding
pub fn verify_secp256r1_signature(
    env: &Env,
    signature_payload: &Hash<32>,
    public_key: &BytesN<65>,
    signature: Secp256r1Signature,
) {
    let Secp256r1Signature {
        mut authenticator_data,
        client_data_json,
        signature,
    } = signature;

    authenticator_data.extend_from_array(&env.crypto().sha256(&client_data_json).to_array());

    env.crypto().secp256r1_verify(
        public_key,
        &env.crypto().sha256(&authenticator_data),
        &signature,
    );

    // Parse the client data JSON, extracting the base64url-encoded challenge.
    let client_data_json = client_data_json.to_buffer::<1024>();
    let client_data_json = client_data_json.as_slice();
    let (client_data_json, _): (ClientDataJson, _) =
        serde_json_core::de::from_slice(client_data_json)
            .unwrap_or_else(|_| panic_with_error!(env, AccountError::JsonParseError));

    // Build what the base64url challenge is expecting.
    let mut expected_challenge = [0u8; 43];

    base64_url::encode(&mut expected_challenge, &signature_payload.to_array());

    // Verify the challenge inside the signed client data JSON matches our expected payload.
    if client_data_json.challenge.as_bytes() != expected_challenge {
        panic_with_error!(env, AccountError::ClientDataJsonChallengeIncorrect)
    }
}
