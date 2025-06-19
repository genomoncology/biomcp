# GenomOncology Remote MCP

**Privacy Policy**
**Version 1.2 – Effective June 18, 2025**

## 1. Data We Collect

| Type                      | Examples                                 | Source               | Storage        |
| ------------------------- | ---------------------------------------- | -------------------- | -------------- |
| **Account**               | Google user ID, email, display name      | From Google OAuth    | BigQuery       |
| **Queries**               | Prompts, timestamps                      | User input           | BigQuery       |
| **Operational**           | IP address, user-agent                   | Automatic            | Temporary only |
| **Usage**                 | Token counts, latency, model performance | Derived metrics      | Aggregated     |
| **Third-Party Responses** | API responses from PubMed, bioRxiv, etc. | Third-party services | Not stored     |

We do **not** collect sensitive health or demographic information.

---

## 2. How We Use It

- Authenticate and secure the service
- Improve quality, accuracy, and speed of model output
- Analyze aggregate usage for insights
- Monitor third-party API performance (without storing responses)
- Comply with laws

---

## 3. Legal Basis (GDPR/UK)

- **Contractual necessity** (Art. 6(1)(b) GDPR)
- **Legitimate interests** (Art. 6(1)(f))
- **Consent**, where applicable

---

## 4. Who We Share With

- **Google Cloud / Cloudflare** – Hosting & Auth
- **API providers** – e.g., PubMed, bioRxiv
  - Your queries are transmitted to these services
  - We do not control their data retention practices
  - We do not store third-party responses
- **Analytics tools** – e.g., BigQuery
- **Authorities** – if required by law

We **do not sell** your personal data.

---

## 5. Third-Party Data Handling

When you use the Service:

- Your queries may be sent to third-party APIs (PubMed, bioRxiv, TCGA, 1000 Genomes)
- These services have their own privacy policies and data practices
- We use third-party responses to generate output but do not store them
- Third parties may independently retain query data per their policies
- Only your username and queries are stored in our systems

---

## 6. Cookies

We use only **Google OAuth** session cookies.
No additional tracking cookies are set.

---

## 7. Data Retention

- **BigQuery storage** (usernames & queries): Retained indefinitely
- **Operational data** (IP, user-agent): Not retained
- **Third-party responses**: Not stored
- **Aggregated metrics**: Retained indefinitely
- **Account Username**: Retained until deletion requested

---

## 8. Security

- All data encrypted in transit (TLS 1.3)
- Least-privilege access enforced via IAM
- Username and query data stored in BigQuery with strict access control
- Operational data (IP, user-agent) processed but not retained
- **Incident Response**: Security incidents investigated within 24 hours
- **Breach Notification**: Users notified within 72 hours of confirmed breach
- **Security Audits**: Annual third-party security assessments
- **Vulnerability Reporting**: See our [SECURITY.md](https://github.com/genomoncology/biomcp/blob/main/docs/biomcp-security.md)

---

## 9. International Transfers

Data is stored in **Google Cloud's `us-central1`**.
Transfers from the EU/UK rely on **SCCs**.

---

## 10. Your Rights

Depending on your location, you may request to:

- Access, correct, or delete your data
- Restrict or object to processing
- Port your data
- File a complaint (EEA/UK)
- Opt out (California residents)

**Data Export**:

- Available in JSON or CSV format
- Requests fulfilled within 30 days
- Includes: account info, queries, timestamps
- Excludes: operational data, third-party responses, aggregated metrics

Email: **privacy@genomoncology.com**

---

## 11. Children's Privacy

The Service is not intended for use by anyone under **16 years old**.

---

## 12. Policy Changes

We will update this document at `/privacy` with an updated Effective Date.
Material changes will be announced by email.
Version history maintained at: [github.com/genomoncology/biomcp/blob/main/docs/biomcp-privacy.md](https://github.com/genomoncology/biomcp/blob/main/docs/biomcp-privacy.md)

---

## 13. Contact

**Data Protection Officer**
📧 **dpo@genomoncology.com**
📮 GenomOncology LLC – Privacy Office
1138 West 9th Street, Suite 400
Cleveland, OH 44113
