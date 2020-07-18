use crate::crypto::MuxedAccount;
use crate::error::{Error, Result};
use crate::operations::Operation;
use crate::xdr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BumpSequenceOperation {
    source_account: Option<MuxedAccount>,
    bump_to: i64,
}

#[derive(Debug)]
pub struct BumpSequenceOperationBuilder {
    source_account: Option<MuxedAccount>,
    bump_to: Option<i64>,
}

impl BumpSequenceOperation {
    /// Retrieves the operation source account.
    pub fn source_account(&self) -> &Option<MuxedAccount> {
        &self.source_account
    }

    /// Retrieves a reference to the operation source account.
    pub fn source_account_mut(&mut self) -> &mut Option<MuxedAccount> {
        &mut self.source_account
    }

    /// Retrieves the operatino bump to.
    pub fn bump_to(&self) -> &i64 {
        &self.bump_to
    }

    /// Retrieves the operatino bump to.
    pub fn bump_to_mut(&mut self) -> &mut i64 {
        &mut self.bump_to
    }

    /// Returns the xdr operation body.
    pub fn to_xdr_operation_body(&self) -> Result<xdr::OperationBody> {
        let bump_to = xdr::SequenceNumber::new(xdr::Int64::new(self.bump_to));
        let inner = xdr::BumpSequenceOp { bump_to };
        Ok(xdr::OperationBody::BumpSequence(inner))
    }

    /// Creates from the xdr operation body.
    pub fn from_xdr_operation_body(
        source_account: Option<MuxedAccount>,
        x: &xdr::BumpSequenceOp,
    ) -> Result<BumpSequenceOperation> {
        let bump_to = x.bump_to.value.value;
        Ok(BumpSequenceOperation {
            source_account,
            bump_to,
        })
    }
}

impl BumpSequenceOperationBuilder {
    pub fn new() -> BumpSequenceOperationBuilder {
        BumpSequenceOperationBuilder {
            source_account: None,
            bump_to: None,
        }
    }

    pub fn with_source_account<S>(mut self, source: S) -> BumpSequenceOperationBuilder
    where
        S: Into<MuxedAccount>,
    {
        self.source_account = Some(source.into());
        self
    }

    pub fn with_bump_to(mut self, bump_to: i64) -> BumpSequenceOperationBuilder {
        self.bump_to = Some(bump_to);
        self
    }

    pub fn build(self) -> Result<Operation> {
        let bump_to = self
            .bump_to
            .ok_or_else(|| Error::InvalidOperation("missing bump sequence bump to".to_string()))?;

        if bump_to < 0 {
            return Err(Error::InvalidOperation(
                "bump sequence bump to must be non negative".to_string(),
            ));
        }

        Ok(Operation::BumpSequence(BumpSequenceOperation {
            source_account: self.source_account,
            bump_to,
        }))
    }
}

#[cfg(test)]
mod tests {
    use crate::account::{AccountFlags, DataValue, TrustLineFlags};
    use crate::amount::{Amount, Price, Stroops};
    use crate::asset::{Asset, CreditAssetType};
    use crate::crypto::KeyPair;
    use crate::memo::Memo;
    use crate::network::Network;
    use crate::operations::Operation;
    use crate::time_bounds::TimeBounds;
    use crate::transaction::{Transaction, TransactionEnvelope, MIN_BASE_FEE};
    use crate::xdr::{XDRDeserialize, XDRSerialize};
    use std::str::FromStr;

    fn keypair0() -> KeyPair {
        // GDQNY3PBOJOKYZSRMK2S7LHHGWZIUISD4QORETLMXEWXBI7KFZZMKTL3
        KeyPair::from_secret_seed("SBPQUZ6G4FZNWFHKUWC5BEYWF6R52E3SEP7R3GWYSM2XTKGF5LNTWW4R")
            .unwrap()
    }

    fn keypair1() -> KeyPair {
        // GAS4V4O2B7DW5T7IQRPEEVCRXMDZESKISR7DVIGKZQYYV3OSQ5SH5LVP
        KeyPair::from_secret_seed("SBMSVD4KKELKGZXHBUQTIROWUAPQASDX7KEJITARP4VMZ6KLUHOGPTYW")
            .unwrap()
    }

    fn keypair2() -> KeyPair {
        // GB7BDSZU2Y27LYNLALKKALB52WS2IZWYBDGY6EQBLEED3TJOCVMZRH7H
        KeyPair::from_secret_seed("SBZVMB74Z76QZ3ZOY7UTDFYKMEGKW5XFJEB6PFKBF4UYSSWHG4EDH7PY")
            .unwrap()
    }

    #[test]
    fn test_bump_sequence() {
        let kp = keypair0();
        let op = Operation::new_bump_sequence()
            .with_bump_to(123)
            .build()
            .unwrap();

        let mut tx = Transaction::builder(kp.public_key().clone(), 3556091187167235, MIN_BASE_FEE)
            .add_operation(op)
            .to_transaction()
            .unwrap();
        tx.sign(&kp, &Network::new_test());
        let envelope = tx.to_envelope();
        let xdr = envelope.xdr_base64().unwrap();
        let expected = "AAAAAgAAAADg3G3hclysZlFitS+s5zWyiiJD5B0STWy5LXCj6i5yxQAAAGQADKI/AAAAAwAAAAAAAAAAAAAAAQAAAAAAAAALAAAAAAAAAHsAAAAAAAAAAeoucsUAAABAFjXV5orPOkYP+zKGyNKWNJPkZ1UG2n7zyj33W5LHlx1LkD+8vLtB8/GyamKUs7qThchbHdRS9lSBUnvqNkNeCg==";
        assert_eq!(expected, xdr);
        let back = TransactionEnvelope::from_xdr_base64(&xdr).unwrap();
        assert_eq!(envelope, back);
    }

    #[test]
    fn test_bump_sequence_with_source() {
        let kp = keypair0();
        let kp1 = keypair1();
        let op = Operation::new_bump_sequence()
            .with_source_account(kp1.public_key().clone())
            .with_bump_to(i64::MAX)
            .build()
            .unwrap();

        let mut tx = Transaction::builder(kp.public_key().clone(), 3556091187167235, MIN_BASE_FEE)
            .add_operation(op)
            .to_transaction()
            .unwrap();
        tx.sign(&kp, &Network::new_test());
        let envelope = tx.to_envelope();
        let xdr = envelope.xdr_base64().unwrap();
        let expected = "AAAAAgAAAADg3G3hclysZlFitS+s5zWyiiJD5B0STWy5LXCj6i5yxQAAAGQADKI/AAAAAwAAAAAAAAAAAAAAAQAAAAEAAAAAJcrx2g/Hbs/ohF5CVFG7B5JJSJR+OqDKzDGK7dKHZH4AAAALf/////////8AAAAAAAAAAeoucsUAAABAvmRGh/Fe460s9zn2gNu6onhD76G7AL7M3KQ9Ps67y92kQUz0Aw8leVvjAkAvuapExekSc1iMRAkS0uszaQIRCA==";
        assert_eq!(expected, xdr);
        let back = TransactionEnvelope::from_xdr_base64(&xdr).unwrap();
        assert_eq!(envelope, back);
    }
}