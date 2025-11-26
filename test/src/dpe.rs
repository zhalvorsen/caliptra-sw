use dpe::{
    commands::{CertifyKeyCommand, Command, DeriveContextCmd, DeriveContextFlags, SignCommand},
    response::{
        CertifyKeyMldsaExternalMu87Resp, CertifyKeyP384Resp, CertifyKeyResp,
        DeriveContextExportedCdiResp, DeriveContextResp, DpeErrorCode, GetCertificateChainResp,
        GetProfileResp, NewHandleResp, Response, ResponseHdr, SignMlDsaResp, SignP384Resp,
        SignResp,
    },
};
use zerocopy::TryFromBytes;

fn check_dpe_status(resp_bytes: &[u8], expected_status: DpeErrorCode) {
    if let Ok(&ResponseHdr { status, .. }) =
        ResponseHdr::try_ref_from_bytes(&resp_bytes[..core::mem::size_of::<ResponseHdr>()])
    {
        if status != expected_status.get_error_code() {
            panic!("Unexpected DPE Status: 0x{:X}", status);
        }
    }
}

pub fn parse_dpe_response(dpe_cmd: &mut Command, resp_bytes: &[u8]) -> Response {
    // Peek response header so we can panic with an error code in case the command failed.
    check_dpe_status(resp_bytes, DpeErrorCode::NoError);

    match dpe_cmd {
        Command::CertifyKey(CertifyKeyCommand::P384(_)) => Response::CertifyKey(
            CertifyKeyResp::P384(CertifyKeyP384Resp::try_read_from_bytes(resp_bytes).unwrap()),
        ),
        Command::CertifyKey(CertifyKeyCommand::ExternalMu87(_)) => {
            Response::CertifyKey(CertifyKeyResp::MldsaExternalMu87(
                CertifyKeyMldsaExternalMu87Resp::try_read_from_bytes(resp_bytes).unwrap(),
            ))
        }
        Command::DeriveContext(DeriveContextCmd { flags, .. })
            if flags.contains(DeriveContextFlags::EXPORT_CDI) =>
        {
            Response::DeriveContextExportedCdi(
                DeriveContextExportedCdiResp::try_read_from_bytes(resp_bytes).unwrap(),
            )
        }
        Command::DeriveContext(_) => {
            Response::DeriveContext(DeriveContextResp::try_read_from_bytes(resp_bytes).unwrap())
        }
        Command::GetCertificateChain(_) => Response::GetCertificateChain(
            GetCertificateChainResp::try_read_from_bytes(resp_bytes).unwrap(),
        ),
        Command::DestroyCtx(_) => {
            Response::DestroyCtx(ResponseHdr::try_read_from_bytes(resp_bytes).unwrap())
        }
        Command::GetProfile => {
            Response::GetProfile(GetProfileResp::try_read_from_bytes(resp_bytes).unwrap())
        }
        Command::InitCtx(_) => {
            Response::InitCtx(NewHandleResp::try_read_from_bytes(resp_bytes).unwrap())
        }
        Command::RotateCtx(_) => {
            Response::RotateCtx(NewHandleResp::try_read_from_bytes(resp_bytes).unwrap())
        }
        Command::Sign(SignCommand::P384(_)) => Response::Sign(SignResp::P384(
            SignP384Resp::try_read_from_bytes(resp_bytes).unwrap(),
        )),
        Command::Sign(SignCommand::ExternalMu87(_)) => Response::Sign(SignResp::MlDsa(
            SignMlDsaResp::try_read_from_bytes(resp_bytes).unwrap(),
        )),
    }
}
