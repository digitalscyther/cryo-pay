import React, {useEffect} from "react";
import {Row, Col} from "react-bootstrap";
import {apiUrl, NETWORKS} from "../../utils";
import Section from "./Section";
import Sidebar from "./Sidebar";

const OverviewContent = () => (
    <>
        <p>
            Welcome to the documentation for our cutting-edge billing gateway
            application! This platform enables you to create, manage, and process
            invoices seamlessly through blockchain technology. With features like
            real-time notifications, multi-network support, and smart contract
            integration, our application offers an efficient, secure, and
            user-friendly experience.
        </p>
        <p>
            Whether you are a developer integrating with our API, a client
            monitoring payments, or a seller creating invoices, this
            documentation will guide you through every step.
        </p>
    </>
)

const LimitsContent = () => (
    <>
        <h5>Invoice Creation Limits</h5>
        <p>
            To ensure smooth operations, there are limits on the number of
            invoices you can create:
        </p>
        <ul>
            <li>
                <strong>Web Users:</strong> Up to 3 invoices per day.
            </li>
            <li>
                <strong>API Users:</strong> Up to 10 invoices per day.
            </li>
        </ul>
        <p>
            If you need higher limits, please go to your user settings
            (available for authorized users only) and select a subscription plan.
        </p>
    </>
)

const CreateInvoiceContent = () => (
    <>
        <h5>Steps to Create an Invoice</h5>
        <ol>
            <li>
                Log in to your account (optional, but recommended for advanced
                features).
            </li>
            <li>Go to the "Dashboard" and select "Create Invoice."</li>
            <li>
                Fill in the invoice details:
                <ul>
                    <li>
                        <strong>Amount:</strong> The payment amount in the preferred
                        currency.
                    </li>
                    <li>
                        <strong>Blockchain Networks:</strong> Select the supported
                        blockchain networks for the payment (e.g., Ethereum, Binance
                        Smart Chain).
                    </li>
                    <li>
                        <strong>Recipient Details:</strong> Provide the necessary
                        information.
                    </li>
                </ul>
            </li>
            <li>Click "Submit" to generate the invoice.</li>
        </ol>
        <p>
            After creating an invoice, you can share it with the recipient. The
            invoice will be visible on your dashboard for easy tracking.
        </p>
    </>
)

const NotificationsContent = () => (
    <>
        <h5>Stay Informed</h5>
        <p>
            The platform keeps you updated on invoice statuses through two
            notification channels:
        </p>
        <ul>
            <li>
                <strong>Email Notifications:</strong> Get notified instantly when
                invoices are paid. (Note: Email notifications require a
                subscription.)
            </li>
            <li>
                <strong>Telegram Bot:</strong> Receive real-time payment updates
                and manage invoices through our integrated Telegram bot.
            </li>
        </ul>
        <p>
            To enable notifications, link your Telegram account in the
            "Settings" section.
        </p>
    </>
)

const MakingPaymentsContent = () => (
    <>
        <h5>How to Pay an Invoice</h5>
        <ol>
            <li>Open the invoice link shared by the seller.</li>
            <li>
                Connect your MetaMask wallet to the supported blockchain network
                listed in the invoice.
            </li>
            <li>Click "Pay Invoice" and confirm the payment in MetaMask.</li>
            <li>
                Wait for the transaction to be confirmed on the blockchain (30 seconds – 2
                minutes on average).
            </li>
            <li>
                Payment status will update automatically. Notifications will be
                sent to seller upon completion.
            </li>
        </ol>
        <p>
            <strong>Note:</strong> Avoid paying the same invoice twice, as the
            system processes payments reliably. Double-check before initiating a
            transaction.
        </p>
    </>
)

const WebhooksContent = () => (
    <>
        <h5>Webhook Notifications</h5>
        <p>
            When an invoice is paid, Cryo Pay sends an HTTP <code>POST</code> to each
            configured webhook URL. Webhooks are managed in the <em>Settings</em> section
            (requires account login).
        </p>

        <h6>Payload</h6>
        <pre>{`{
  "id": "b92d6367-6bf1-49b8-8180-d7fb79d7c75b",
  "paid_at": "2025-02-14T23:05:38",
  "status": "SUCCESS"
}`}</pre>
        <p><code>paid_at</code> is an ISO 8601 datetime string (may be <code>null</code> in edge cases).</p>

        <h6>Request Headers</h6>
        <ul>
            <li><code>Content-Type: application/json</code></li>
            <li><code>X-Webhook-Timestamp: &lt;unix seconds&gt;</code> — always present</li>
            <li><code>X-Signature-256: &lt;hex&gt;</code> — present when a secret is set</li>
        </ul>

        <h6>Signature Verification</h6>
        <p>
            The signature is <code>{"HMAC-SHA256(secret, \"{timestamp}.{raw_json_body}\")"}</code>.
            The signed string is the timestamp, a literal dot, then the exact JSON body bytes.
            Always use a constant-time comparison to prevent timing attacks.
        </p>

        <strong>Python</strong>
        <pre>{`import hmac, hashlib

def verify(secret: str, timestamp: str, body: bytes, signature: str) -> bool:
    msg = f"{timestamp}.".encode() + body
    expected = hmac.new(secret.encode(), msg, hashlib.sha256).hexdigest()
    return hmac.compare_digest(expected, signature)

# In your request handler:
timestamp = request.headers["X-Webhook-Timestamp"]
signature = request.headers["X-Signature-256"]
verify(WEBHOOK_SECRET, timestamp, request.body, signature)`}</pre>

        <strong>Node.js</strong>
        <pre>{`const crypto = require('crypto');

function verify(secret, timestamp, body, signature) {
  const msg = \`\${timestamp}.\${body}\`;
  const expected = crypto.createHmac('sha256', secret).update(msg).digest('hex');
  return crypto.timingSafeEqual(Buffer.from(expected), Buffer.from(signature));
}

// body must be the raw request body string, not parsed JSON
const timestamp = req.headers['x-webhook-timestamp'];
const signature = req.headers['x-signature-256'];
verify(WEBHOOK_SECRET, timestamp, rawBody, signature);`}</pre>

        <p>
            <strong>Notes:</strong>{" "}
            Max 2 webhooks per account. The URL must be publicly reachable (localhost and private IPs are blocked)
            and return 2xx on a test <code>POST</code> sent at creation time.
            Delivery is retried up to 2 times on failure.
            Webhooks created without a secret skip signature generation (legacy mode).
        </p>
    </>
)

const FaqContent = () => (
    <>
        <h5>Frequently Asked Questions</h5>
        <ul>
            <li>
                <strong>What if my payment fails?</strong>
                <p>
                    If a payment fails, verify your wallet balance and network
                    connection. Retry the transaction from the invoice link.
                </p>
            </li>
            <li>
                <strong>Can I modify an invoice after creating it?</strong>
                <p>
                    No, invoices cannot be edited after creation. If necessary, you
                    can delete unpaid invoices and create new ones with the correct
                    details.
                </p>
            </li>
            <li>
                <strong>How can I receive notifications?</strong>
                <p>
                    Enable email notifications in
                    "Settings." Link your Telegram account to activate bot
                    notifications.
                </p>
            </li>
        </ul>
    </>
)

const ApiReferenceContent = () => (
    <>
        <p>Browse and try every endpoint interactively:</p>
        <p>
            <a
                href={apiUrl('/swagger-ui/')}
                target="_blank"
                rel="noreferrer"
                className="btn btn-primary"
            >
                Open Swagger UI ↗
            </a>
        </p>
        <hr />
        <h5>Authentication</h5>
        <p>
            <strong>API key</strong> (Bearer token) — works for <code>/payment/*</code> and{' '}
            <code>/blockchain/*</code> routes. Obtain from Settings → API Keys.
        </p>
        <p>
            <strong>Session cookie</strong> — required for all <code>/user/*</code> routes.
            Log in via the web UI; the cookie is set automatically.
        </p>
        <hr />
        <h5>Supported Networks</h5>
        <table className="table table-bordered table-sm">
            <thead>
                <tr><th>Chain ID</th><th>Network</th></tr>
            </thead>
            <tbody>
                {Object.values(NETWORKS).sort((a, b) => a.order - b.order).map(n => (
                    <tr key={n.id}><td>{n.id}</td><td>{n.name}</td></tr>
                ))}
            </tbody>
        </table>
    </>
);

const Documentation = () => {
    useEffect(() => {
        const hash = window.location.hash;
        if (hash) {
            const element = document.getElementById(hash.substring(1)); // Remove the '#' character
            if (element) {
                element.scrollIntoView({behavior: "smooth", block: "start"});
            }
        }
    }, []);

    const sections = [
        {id: "overview", title: "Overview", content: (<OverviewContent/>)},
        {id: "limits", title: "Limits", content: (<LimitsContent/>)},
        {id: "creating-invoices", title: "Creating Invoices", content: (<CreateInvoiceContent/>)},
        {id: "payments", title: "Making Payments", content: (<MakingPaymentsContent/>)},
        {id: "notifications", title: "Notifications", content: (<NotificationsContent/>)},
        {id: "webhooks", title: "Webhooks", content: (<WebhooksContent/>)},
        {id: "faq", title: "FAQ", content: (<FaqContent/>)},
        {id: "api-reference", title: "API Reference", content: (<ApiReferenceContent/>)},
    ];

    return (<>
        <Row>
            <Col>
                <h2 className="m-3">Documentation</h2>
            </Col>
        </Row>
        <Row className="mt-2">
            {/* Sidebar */}
            <Col id="doc-nav" md={2} className="bg-dark text-white p-3">
                <Sidebar sections={sections}/>
            </Col>

            {/* Content Area */}
            <Col className="mt-md-0 mt-3" md={10}>
                {sections.map((section) => (<Section key={section.id} section={section}/>))}
            </Col>
        </Row>
    </>);
};

export default Documentation;