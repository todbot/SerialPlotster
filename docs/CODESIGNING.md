# Code Signing Setup

Variables are stored as repository secrets in **Settings → Secrets and variables → Actions**.

---

## macOS — Signing (`sign_macos`)

| Secret | Description |
|---|---|
| `APPLE_CERTIFICATE` | Base64-encoded Developer ID Application certificate exported as a `.p12` file. Export from Keychain Access: right-click the cert → Export → Personal Information Exchange (`.p12`). Encode with `base64 -i cert.p12 \| pbcopy`. |
| `APPLE_CERTIFICATE_PASSWORD` | Password chosen when exporting the `.p12`. |
| `APPLE_SIGNING_IDENTITY` | The full name of the certificate as it appears in Keychain Access, e.g. `Developer ID Application: Your Name (ABCD1234EF)`. Run `security find-identity -v -p codesigning` to list candidates. |

## macOS — Notarization (`notarize_macos`)

Notarization requires signing to also be enabled. Apple scans the app and staples a ticket so Gatekeeper passes without an internet connection.

| Secret | Description |
|---|---|
| `APPLE_ID` | The Apple ID email address (e.g. `you@example.com`) associated with your Apple Developer account. |
| `APPLE_PASSWORD` | An **app-specific password** for that Apple ID — not your account password. Generate one at [appleid.apple.com](https://appleid.apple.com) under Sign-In and Security → App-Specific Passwords. |
| `APPLE_TEAM_ID` | Your 10-character Apple Developer Team ID, e.g. `ABCD1234EF`. Visible at [developer.apple.com/account](https://developer.apple.com/account) under Membership Details. |

---

## Windows — Azure Trusted Signing (`sign_windows`)

Azure Trusted Signing issues short-lived certificates on demand; no `.pfx` file is stored. Signing is performed by `signtool.exe` via the `Azure.CodeSigning.Dlib.dll` plugin, which authenticates using a service principal.

### Azure setup steps

1. Create a **Trusted Signing account** and a **Certificate Profile** (Public Trust or Private Trust) in the Azure Portal.
2. Create an **App Registration** (service principal) in Azure Active Directory.
3. In the Trusted Signing account's **Access Control (IAM)**, assign the service principal the **Trusted Signing Certificate Profile Signer** role.
4. Create a **client secret** for the app registration and copy it immediately.

### Secrets

| Secret | Description |
|---|---|
| `AZURE_TENANT_ID` | Azure Active Directory tenant ID (a UUID). Found in Azure Portal → Azure Active Directory → Overview. |
| `AZURE_CLIENT_ID` | Application (client) ID of the app registration (a UUID). Found in the app registration's Overview page. |
| `AZURE_CLIENT_SECRET` | Client secret value created under the app registration's Certificates & Secrets. Only shown once at creation time. |
| `AZURE_TRUSTED_SIGNING_ENDPOINT` | Regional endpoint for your Trusted Signing account, e.g. `https://eus.codesigning.azure.net`. Shown on the Trusted Signing account's Overview page in the Azure Portal. |
| `AZURE_TRUSTED_SIGNING_ACCOUNT` | The name of your Trusted Signing account resource. |
| `AZURE_TRUSTED_SIGNING_CERT_PROFILE` | The name of the Certificate Profile within that account. |
