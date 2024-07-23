use std::str::FromStr;

use crate::operation::{SignedOperation, SignedPLCOperation};
use crate::{operation::PLCOperationType, util::op_from_json};
use crate::{PLCError, PLCOperation};
use chrono::{DateTime, Local, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AuditLog {
    pub cid: String,
    pub created_at: NaiveDateTime,
    pub did: String,
    pub nullified: bool,
    pub operation: PLCOperation,
}

impl AuditLog {
    pub fn from_json(json: &str) -> Result<Self, PLCError> {
        let json: serde_json::Value =
            serde_json::from_str(json).map_err(|e| PLCError::Other(e.into()))?;
        let op = op_from_json(
            &serde_json::to_string(json.get("operation").unwrap())
                .map_err(|e| PLCError::Other(e.into()))?,
        )?;
        Ok(AuditLog {
            cid: json.get("cid").unwrap().as_str().unwrap().to_string(),
            created_at: DateTime::<Utc>::from_str(json.get("createdAt").unwrap().as_str().unwrap())
                .unwrap()
                .naive_utc(),
            did: json.get("did").unwrap().as_str().unwrap().to_string(),
            nullified: json.get("nullified").unwrap().as_bool().unwrap(),
            operation: op,
        })
    }

    pub fn op_type(&self) -> PLCOperationType {
        match &self.operation {
            PLCOperation::SignedGenesis(_) => PLCOperationType::Operation,
            PLCOperation::UnsignedGenesis(_) => PLCOperationType::Operation,
            PLCOperation::SignedPLC(op) => op.unsigned.type_.clone(),
            PLCOperation::UnsignedPLC(op) => op.type_.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DIDAuditLogs(Vec<AuditLog>);

impl DIDAuditLogs {
    pub fn from_json(json: &str) -> Result<Self, PLCError> {
        let json: serde_json::Value =
            serde_json::from_str(json).map_err(|e| PLCError::Other(e.into()))?;
        let logs = json
            .as_array()
            .unwrap()
            .iter()
            .map(|log| AuditLog::from_json(&serde_json::to_string(log).unwrap()).unwrap())
            .collect();
        Ok(Self(logs))
    }

    pub fn get_latest(&self) -> Result<String, PLCError> {
        let last_op = self.0.last();
        if last_op.is_none() {
            return Err(PLCError::MisorderedOperation);
        }
        let mut last_op = last_op.unwrap();
        if last_op.op_type() == PLCOperationType::Tombstone {
            if self.0.len() == 1 {
                return Err(PLCError::InvalidOperation);
            }
            last_op = &self.0[self.0.len() - 2];
        }
        Ok(last_op.cid.clone())
    }

    pub fn assure_valid(&self, proposed: SignedPLCOperation) -> Result<bool, PLCError> {
        let cid = match &proposed.unsigned.prev {
            Some(cid) => cid,
            None => return Err(PLCError::MisorderedOperation),
        };
        let index_of_prev = self.0.iter().position(|log| log.cid == cid.to_string());

        if index_of_prev.is_none() {
            return Err(PLCError::MisorderedOperation);
        }

        let ops_in_history = self.0[0..index_of_prev.unwrap()].to_vec();
        let nullified = self.0[index_of_prev.unwrap()..self.0.len() - 1].to_vec();

        let last_op = ops_in_history.last();
        if last_op.is_none() {
            return Err(PLCError::MisorderedOperation);
        }
        let last_op = last_op.unwrap();
        if last_op.op_type() == PLCOperationType::Tombstone {
            return Err(PLCError::MisorderedOperation);
        }

        let last_op_normalized: SignedPLCOperation = match &last_op.operation {
            PLCOperation::SignedGenesis(op) => op.normalize().unwrap().into(),
            PLCOperation::SignedPLC(op) => op.clone(),
            _ => {
                unreachable!()
            }
        };

        // No nullification is involved
        let first_nullified = nullified.first();
        if first_nullified.is_none() {
            match last_op_normalized.verify_sig(None) {
                Ok(_) => {
                    return Ok(true);
                }
                Err(_) => {
                    return Err(PLCError::InvalidSignature);
                }
            }
        }
        let first_nullified = first_nullified.unwrap();

        let (_, disputed_key) = match last_op_normalized.verify_sig(None) {
            Ok(result) => {
                if !result.0 {
                    return Err(PLCError::InvalidSignature);
                }
                result
            }
            Err(_) => {
                return Err(PLCError::InvalidSignature);
            }
        };
        let disputed_key = disputed_key.unwrap();

        let signer_index = last_op_normalized
            .unsigned
            .rotation_keys
            .iter()
            .position(|key| key == &disputed_key)
            .unwrap();
        let more_powerful_keys = last_op_normalized
            .unsigned
            .rotation_keys
            .split_at(signer_index)
            .1
            .to_vec();

        match proposed.verify_sig(Some(more_powerful_keys)) {
            Ok(result) => {
                if !result.0 {
                    return Err(PLCError::InvalidSignature);
                }
            }
            Err(_) => {
                return Err(PLCError::InvalidSignature);
            }
        }

        if nullified.len() > 0 {
            const RECOVERY_WINDOW: i64 = 72 * 60 * 60;
            let local = Local::now().naive_utc();
            let time_lapsed = local - first_nullified.created_at;
            if time_lapsed.num_seconds() > RECOVERY_WINDOW {
                return Err(PLCError::LateRecovery);
            }
        }

        return Ok(true);
    }

    pub fn last(&self) -> Option<&AuditLog> {
        self.0.last()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_AUDIT_LOG: &str = "[{\"did\":\"did:plc:z72i7hdynmk6r22z27h6tvur\",\"operation\":{\"sig\":\"9NuYV7AqwHVTc0YuWzNV3CJafsSZWH7qCxHRUIP2xWlB-YexXC1OaYAnUayiCXLVzRQ8WBXIqF-SvZdNalwcjA\",\"prev\":null,\"type\":\"plc_operation\",\"services\":{\"atproto_pds\":{\"type\":\"AtprotoPersonalDataServer\",\"endpoint\":\"https://bsky.social\"}},\"alsoKnownAs\":[\"at://bluesky-team.bsky.social\"],\"rotationKeys\":[\"did:key:zQ3shhCGUqDKjStzuDxPkTxN6ujddP4RkEKJJouJGRRkaLGbg\",\"did:key:zQ3shpKnbdPx3g3CmPf5cRVTPe1HtSwVn5ish3wSnDPQCbLJK\"],\"verificationMethods\":{\"atproto\":\"did:key:zQ3shXjHeiBuRCKmM36cuYnm7YEMzhGnCmCyW92sRJ9pribSF\"}},\"cid\":\"bafyreigp6shzy6dlcxuowwoxz7u5nemdrkad2my5zwzpwilcnhih7bw6zm\",\"nullified\":false,\"createdAt\":\"2023-04-12T04:53:57.057Z\"},{\"did\":\"did:plc:z72i7hdynmk6r22z27h6tvur\",\"operation\":{\"sig\":\"1mEWzRtFOgeRXH-YCSPTxb990JOXxa__n8Qw6BOKl7Ndm6OFFmwYKiiMqMCpAbxpnGjF5abfIsKc7u3a77Cbnw\",\"prev\":\"bafyreigp6shzy6dlcxuowwoxz7u5nemdrkad2my5zwzpwilcnhih7bw6zm\",\"type\":\"plc_operation\",\"services\":{\"atproto_pds\":{\"type\":\"AtprotoPersonalDataServer\",\"endpoint\":\"https://bsky.social\"}},\"alsoKnownAs\":[\"at://bsky.app\"],\"rotationKeys\":[\"did:key:zQ3shhCGUqDKjStzuDxPkTxN6ujddP4RkEKJJouJGRRkaLGbg\",\"did:key:zQ3shpKnbdPx3g3CmPf5cRVTPe1HtSwVn5ish3wSnDPQCbLJK\"],\"verificationMethods\":{\"atproto\":\"did:key:zQ3shXjHeiBuRCKmM36cuYnm7YEMzhGnCmCyW92sRJ9pribSF\"}},\"cid\":\"bafyreihmuvr3frdvd6vmdhucih277prdcfcezf67lasg5oekxoimnunjoq\",\"nullified\":false,\"createdAt\":\"2023-04-12T17:26:46.468Z\"},{\"did\":\"did:plc:z72i7hdynmk6r22z27h6tvur\",\"operation\":{\"sig\":\"OoDJihYhLUEWp2MGiAoCN1sRj9cgUEqNjZe6FIOePB8Ugp-IWAZplFRm-pU-fbYSpYV1_tQ9Gx8d_PR9f3NBAg\",\"prev\":\"bafyreihmuvr3frdvd6vmdhucih277prdcfcezf67lasg5oekxoimnunjoq\",\"type\":\"plc_operation\",\"services\":{\"atproto_pds\":{\"type\":\"AtprotoPersonalDataServer\",\"endpoint\":\"https://bsky.social\"}},\"alsoKnownAs\":[\"at://bsky.app\"],\"rotationKeys\":[\"did:key:zQ3shhCGUqDKjStzuDxPkTxN6ujddP4RkEKJJouJGRRkaLGbg\",\"did:key:zQ3shpKnbdPx3g3CmPf5cRVTPe1HtSwVn5ish3wSnDPQCbLJK\"],\"verificationMethods\":{\"atproto\":\"did:key:zQ3shXjHeiBuRCKmM36cuYnm7YEMzhGnCmCyW92sRJ9pribSF\"}},\"cid\":\"bafyreiexwziulimyiw3qlhpwr2zljk5jtzdp2bgqbgoxuemjsf5a6tan3a\",\"nullified\":false,\"createdAt\":\"2023-06-01T20:05:52.008Z\"},{\"did\":\"did:plc:z72i7hdynmk6r22z27h6tvur\",\"operation\":{\"sig\":\"8Wj9Cf74dZFNKx7oucZSHbBDFOMJ3xx9lkvj5rT9xMErssWYl1D9n4PeGC0mNml7xDG7uoQqZ1JWoApGADUgXg\",\"prev\":\"bafyreiexwziulimyiw3qlhpwr2zljk5jtzdp2bgqbgoxuemjsf5a6tan3a\",\"type\":\"plc_operation\",\"services\":{\"atproto_pds\":{\"type\":\"AtprotoPersonalDataServer\",\"endpoint\":\"https://puffball.us-east.host.bsky.network\"}},\"alsoKnownAs\":[\"at://bsky.app\"],\"rotationKeys\":[\"did:key:zQ3shhCGUqDKjStzuDxPkTxN6ujddP4RkEKJJouJGRRkaLGbg\",\"did:key:zQ3shpKnbdPx3g3CmPf5cRVTPe1HtSwVn5ish3wSnDPQCbLJK\"],\"verificationMethods\":{\"atproto\":\"did:key:zQ3shQo6TF2moaqMTrUZEM1jeuYRQXeHEx4evX9751y2qPqRA\"}},\"cid\":\"bafyreifn4pkect7nymne3sxkdg7tn7534msyxcjkshmzqtijmn3enyxm3q\",\"nullified\":false,\"createdAt\":\"2023-11-09T21:49:10.793Z\"}]";

    #[test]
    fn test_did_audit_log_from_json() {
        let audit_logs = DIDAuditLogs::from_json(TEST_AUDIT_LOG).unwrap();
        assert_eq!(audit_logs.0.len(), 4);
    }
}
