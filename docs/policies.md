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

# Security Policy

## Reporting a Vulnerability

We take the security of biomcp seriously. If you believe you have found a security vulnerability, please report it to us as described below.

### Please do NOT:

- Open a public GitHub issue
- Discuss the vulnerability publicly before it has been addressed

### Please DO:

- Email us at **security@genomoncology.com**
- Include the word "SECURITY" in the subject line
- Provide detailed steps to reproduce the vulnerability
- Include the impact and potential attack scenarios

### What to expect:

- **Acknowledgment**: Within 24 hours
- **Initial Assessment**: Within 72 hours
- **Status Updates**: At least every 5 business days
- **Resolution Target**: Critical issues within 30 days

### Scope

Vulnerabilities in the following areas are in scope:

- Authentication bypass or privilege escalation
- Data exposure or unauthorized access to user queries
- Injection vulnerabilities (SQL, command, etc.)
- Cross-site scripting (XSS) or request forgery (CSRF)
- Denial of service vulnerabilities
- Insecure cryptographic implementations
- Third-party API key exposure

### Out of Scope:

- Vulnerabilities in third-party services (PubMed, bioRxiv, etc.)
- Issues in dependencies with existing patches
- Social engineering attacks
- Physical attacks
- Attacks requiring authenticated admin access

## Disclosure Policy

- We will work with you to understand and validate the issue
- We will prepare a fix and release it as soon as possible
- We will publicly disclose the vulnerability after the fix is released
- We will credit you for the discovery (unless you prefer to remain anonymous)

## Safe Harbor

Any activities conducted in a manner consistent with this policy will be considered authorized conduct, and we will not initiate legal action against you. If legal action is initiated by a third party against you in connection with activities conducted under this policy, we will take steps to make it known that your actions were conducted in compliance with this policy.

## Contact

**Security Team Email**: security@genomoncology.com
**PGP Key**: Available upon request

Thank you for helping keep biomcp and our users safe!

# GenomOncology Remote MCP

**Terms of Service**
**Version 1.2 – Effective June 18, 2025**

> This document applies to the **hosted Remote MCP service** (the "Service") provided by **GenomOncology LLC**.
>
> For use of the **open-source code** available at [https://github.com/genomoncology/biomcp](https://github.com/genomoncology/biomcp), refer to the repository's LICENSE file (e.g., MIT License).

---

## 1. Definitions

| Term                  | Meaning                                                                                                                                                                |
| --------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Service**           | The hosted Model Context Protocol (MCP) instance available via Cloudflare and secured by Google OAuth.                                                                 |
| **User Content**      | Prompts, messages, files, code, or other material submitted by you.                                                                                                    |
| **Output**            | Model-generated text or data produced in response to your User Content.                                                                                                |
| **Personal Data**     | Information that identifies or relates to an identifiable individual, including Google account identifiers and query text.                                             |
| **Commercial Use**    | Any use that directly or indirectly generates revenue, including but not limited to: selling access, integrating into paid products, or using for business operations. |
| **Academic Research** | Non-commercial research conducted by accredited educational institutions for scholarly purposes.                                                                       |

---

## 2. Eligibility & Accounts

You must:

- Be at least 16 years old
- Have a valid Google account
- Not be barred from receiving services under applicable law

Authentication is handled via **Google OAuth**. Keep your credentials secure.

---

## 3. License & Intellectual Property

You are granted a **limited, revocable, non-exclusive, non-transferable** license to use the Service for **internal research and non-commercial evaluation**.

**Permitted Uses:**

- Personal research and learning
- Academic research (with attribution)
- Evaluation for potential commercial licensing
- Open-source development (non-commercial)

**Prohibited Commercial Uses:**

- Reselling or redistributing Service access
- Integration into commercial products/services
- Use in revenue-generating operations
- Commercial data analysis or insights

For commercial licensing inquiries, contact: **licensing@genomoncology.com**

We retain all rights in the Service and its software.
You retain ownership of your User Content, but grant us a royalty-free, worldwide license to use it (and the resulting Output) to provide, secure, and improve the Service.

---

## 4. Acceptable Use & Rate Limits

You **must not**:

1. Violate any law or regulation
2. Reverse-engineer, scrape, or probe the Service or model weights
3. Exceed rate limits or disrupt the Service

**Rate Limits:**

- **Standard tier**: 100 requests per hour, 1000 per day
- **Burst limit**: 10 requests per minute
- **Payload size**: 50KB per request

**Exceeding Limits:**

- First violation: 1-hour suspension
- Repeated violations: Account review and possible termination
- Higher limits available upon request: **api-limits@genomoncology.com**

---

## 5. Privacy, Logging & Improvement

We store **Google user ID**, **email address**, and **query text** with **timestamps** in **Google BigQuery**. This data is analyzed to:

- Operate and secure the Service
- Improve system performance and user experience
- Tune models and develop features
- Generate usage analytics

**Note**: We process but do not retain operational data like IP addresses or user-agents. Third-party API responses are used in real-time but not stored.

See our [Privacy Policy](https://github.com/genomoncology/biomcp/blob/main/docs/biomcp-privacy.md) for details.

---

## 6. Third‑Party Services

The Service queries third-party APIs and knowledge sources (e.g., **PubMed, bioRxiv, TCGA, 1000 Genomes**) to respond to user prompts.

**Important:**

- Your queries are transmitted to these services
- Third-party services have independent terms and privacy policies
- We cannot guarantee their availability, accuracy, or uptime
- Third parties may retain your query data per their policies
- API responses are used to generate output but not stored by us

You acknowledge that third-party content is subject to their respective licenses and terms.

---

## 7. Disclaimers

- **AI Output:** May be inaccurate or biased. **Do not rely on it for medical or legal decisions.**
- **AS‑IS:** The Service is provided _"as is"_ with no warranties or guarantees.
- **Third-Party Content:** We are not responsible for accuracy or availability of third-party data.

---

## 8. Limitation of Liability

To the extent permitted by law, **GenomOncology** is not liable for indirect, incidental, or consequential damages, including:

- Data loss
- Business interruption
- Inaccurate output
- Third-party service failures

---

## 9. Indemnification

You agree to indemnify and hold GenomOncology harmless from any claim resulting from your misuse of the Service.

---

## 10. Termination

We may suspend or terminate access at any time. Upon termination:

- Your license ends immediately
- We retain stored data (username & queries) per our Privacy Policy
- You may request data export within 30 days

---

## 11. Governing Law & Dispute Resolution

These Terms are governed by the laws of **Ohio, USA**.
Disputes will be resolved via binding arbitration in **Cuyahoga County, Ohio**, under **JAMS Streamlined Rules**.

---

## 12. Changes

We may update these Terms by posting to `/terms`.
Material changes will be emailed. Continued use constitutes acceptance.
Version history: [github.com/genomoncology/biomcp/blob/main/docs/biomcp-terms.md](https://github.com/genomoncology/biomcp/blob/main/docs/biomcp-terms.md)

---

## 13. Security & Vulnerability Reporting

Found a security issue? Please report it responsibly:

- Email: **security@genomoncology.com**
- See: [SECURITY.md](https://github.com/genomoncology/biomcp/blob/main/SECURITY.md)

---

## 14. Contact

GenomOncology LLC
1138 West 9th Street, Suite 400
Cleveland, OH 44113
📧 **legal@genomoncology.com**

---

## Appendix A – Acceptable Use Policy (AUP)

- Do not submit illegal, harassing, or hateful content
- Do not generate malware, spam, or scrape personal data
- Respect copyright and IP laws
- Do not attempt to re-identify individuals from model output
- Do not use the Service to process protected health information (PHI)
- Do not submit personally identifiable genetic data
