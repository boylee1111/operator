//  Copyright 2025 RustFS Team
//
//  Licensed under the Apache License, Version 2.0 (the "License");
//  you may not use this file except in compliance with the License.
//  You may obtain a copy of the License at
//
//      http:www.apache.org/licenses/LICENSE-2.0
//
//  Unless required by applicable law or agreed to in writing, software
//  distributed under the License is distributed on an "AS IS" BASIS,
//  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//  See the License for the specific language governing permissions and
//  limitations under the License.

/// Default RustFS container image when neither spec.image nor TENANT_RUSTFS_IMAGE is set.
pub const DEFAULT_RUSTFS_IMAGE: &str = "rustfs/rustfs:1.0.0";

pub(crate) fn get_rustfs_image() -> Option<String> {
    std::env::var("TENANT_RUSTFS_IMAGE").ok()
}

/// Returns the RustFS image to use: spec image > TENANT_RUSTFS_IMAGE env > default.
/// Never returns empty; StatefulSet container.image is required by Kubernetes.
pub(crate) fn get_rustfs_image_or_default(explicit: Option<&String>) -> String {
    explicit
        .cloned()
        .or_else(get_rustfs_image)
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| DEFAULT_RUSTFS_IMAGE.to_string())
}

pub(crate) fn get_rustfs_mount_path() -> Option<String> {
    Some("/data".to_owned())
}

/// Environment variable that selects the mTLS readiness probe mode (`rustfs`, `curl`, or `tcp`).
/// Defaults to `rustfs` (`rustfs health --ready --insecure --timeout 2`), which works in both
/// standard and distroless container images without requiring `curl` or `/bin/sh`.
/// Set `curl` or `tcp` on the operator process or in `tenant.spec.env` when running older RustFS
/// images that predate the built-in `rustfs health` subcommand.
pub const TENANT_MTLS_READINESS_PROBE_ENV: &str = "TENANT_MTLS_READINESS_PROBE";
pub const RUSTFS_MTLS_READINESS_PROBE_ENV: &str = "RUSTFS_MTLS_READINESS_PROBE";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum MtlsReadinessProbeMode {
    #[default]
    Rustfs,
    Curl,
    Tcp,
}

impl MtlsReadinessProbeMode {
    pub(crate) fn parse(value: &str) -> Self {
        let trimmed = value.trim();
        if trimmed.eq_ignore_ascii_case("tcp") || trimmed.eq_ignore_ascii_case("tcpsocket") {
            Self::Tcp
        } else if trimmed.eq_ignore_ascii_case("curl") {
            Self::Curl
        } else {
            Self::Rustfs
        }
    }
}

pub(crate) fn mtls_readiness_probe_mode(
    explicit_env: &[k8s_openapi::api::core::v1::EnvVar],
) -> MtlsReadinessProbeMode {
    if let Some(value) = explicit_env
        .iter()
        .rev()
        .find(|env| {
            env.name == TENANT_MTLS_READINESS_PROBE_ENV
                || env.name == RUSTFS_MTLS_READINESS_PROBE_ENV
        })
        .and_then(|env| env.value.as_deref())
    {
        return MtlsReadinessProbeMode::parse(value);
    }

    std::env::var(TENANT_MTLS_READINESS_PROBE_ENV)
        .ok()
        .or_else(|| std::env::var(RUSTFS_MTLS_READINESS_PROBE_ENV).ok())
        .as_deref()
        .map_or(
            MtlsReadinessProbeMode::Rustfs,
            MtlsReadinessProbeMode::parse,
        )
}
