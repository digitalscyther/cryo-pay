import React, {useEffect} from "react";
import {Alert, Table, Row, Col, Card, Nav} from "react-bootstrap";
import {apiUrl, getFullUrl, NETWORKS} from "../utils";

const Section = ({section}) => (<Card
    className={`mb-4 ${section.notReady ? "bg-secondary text-white" : "bg-light"}`}
>
    <Card.Body>
        <Card.Title id={section.id} className="text-dark">
            {section.title}
        </Card.Title>
        {section.notReady ? (<Card.Text>
            <em>This section is under construction. Please check back later.</em>
        </Card.Text>) : (<>{section.content}</>)}
    </Card.Body>
</Card>);

const Sidebar = ({sections}) => (<Col id="doc-nav" md={3} className="bg-dark text-white p-3">
    <Nav className="flex-column">
        {sections.map((section) => (<Nav.Link
            key={section.id}
            href={`#${section.id}`}
            className={`text-light ${section.notReady ? "not-ready" : ""}`}
            title={section.notReady ? "Coming Soon" : ""}
        >
            {section.title}
            {section.notReady && <small className="ms-2">(Coming Soon)</small>}
        </Nav.Link>))}
    </Nav>
</Col>);

function APIEndpoints({urlInvoices, urlInvoicesInstance}) {
    let invoice_id = "3d9de564-c1b9-4e81-b98d-58160d163002";
    let invoice_url = getFullUrl("/invoices/:invoice_id");
    let invoice_url_example = getFullUrl(`/invoices/${invoice_id}`);
    let callback_url = "https://foo.bar/baz";
    let invoice_url_with_callback_url = getFullUrl(`/invoices/${invoice_id}?callback_url=${callback_url}`);
    let redirect_url = `${callback_url}?invoice_id=${invoice_id}&status=SUCCESS`;

    return (<>
        <Alert variant="primary">
            <h5 className="mb-3">🚀 API Endpoints</h5>
            <p>
                Our API currently supports <strong>invoice management</strong>{" "}
                functionalities. Below are the details of how to integrate and use the
                API securely and effectively.
            </p>
        </Alert>

        <h6>🔒 Authentication</h6>
        <p>
            Access to the API requires an <strong>API key</strong>. Generate your
            API key in the <em>Settings</em> section of your account. Include it in
            the request headers like this:
        </p>
        <pre>{`Authorization: Bearer YOUR_API_KEY`}</pre>
        <p>
            <strong>Note:</strong> API keys are available only to logged-in users.
        </p>

        <h6>📜 Available Invoice Endpoints</h6>
        <Table bordered hover>
            <thead>
            <tr>
                <th>Endpoint</th>
                <th>Method</th>
                <th>Description</th>
            </tr>
            </thead>
            <tbody>
            <tr>
                <td>{urlInvoices}</td>
                <td>GET</td>
                <td>Retrieve a list of invoices with pagination and filtering.</td>
            </tr>
            <tr>
                <td>{urlInvoices}</td>
                <td>POST</td>
                <td>
                    Create a new invoice by specifying the amount, seller address, and
                    supported networks.
                </td>
            </tr>
            <tr>
                <td>{urlInvoicesInstance}</td>
                <td>GET</td>
                <td>Get details of a specific invoice, including ownership information.</td>
            </tr>
            <tr>
                <td>{urlInvoicesInstance}</td>
                <td>DELETE</td>
                <td>Delete an invoice owned by the authenticated user.</td>
            </tr>
            </tbody>
        </Table>

        <h6>📜 Request Payload for Creating an Invoice</h6>
        <p>Here’s how the payload for creating a new invoice looks:</p>
        <pre>
        {`POST ${urlInvoices} HTTP/1.1
  Authorization: Bearer YOUR_API_KEY
  Content-Type: application/json
  
  {
      "amount": "150.00",
      "seller": "0xYourEthereumAddressHere",
      "networks": [10, 42161]
  }`}
      </pre>
        <Alert variant="info">
            <strong>⚠️ Seller:</strong> This should be the Ethereum-based address
            (e.g., <code>0x...</code>) where funds will be accepted.
        </Alert>

        {/* New Section: Accessing and Sharing Invoices */}
        <h6>🔗 Accessing and Sharing Invoices</h6>
        <p>
            Once an invoice is created, it becomes available at a unique, shareable URL. You can find this URL in
            the
            invoice details.
            The format of the URL is:
        </p>
        <pre>{invoice_url}</pre>
        <p>
            For example:
        </p>
        <pre>{invoice_url_example}</pre>
        <p>
            You can share this URL directly with your clients for payment.
        </p>

        {/* New Section: Callback URL for Post-Payment Redirection */}
        <h6>🔄 Post-Payment Redirection with Callback URL</h6>
        <p>
            You can specify a <code>callback_url</code> as a query parameter when sharing the invoice URL.
            After a successful payment, the user will be automatically redirected to this URL. This is useful for
            integrating the payment process with your application's workflow.
        </p>
        <p>
            Example of an invoice URL with a <code>callback_url</code>:
        </p>
        <pre>{invoice_url_with_callback_url}</pre>
        <p>
            In this example, after the payment is completed, the user will be redirected
            to <code>{redirect_url}</code>.
            You can use this to update your system about the payment status, display a custom message to the user,
            or
            trigger other actions in your application.
            Make sure that the provided <code>callback_url</code> is URL-encoded.
        </p>

        <h6>🌐 Supported Networks</h6>
        <Table striped bordered>
            <thead>
            <tr>
                <th>Name</th>
                <th>Network ID</th>
            </tr>
            </thead>
            <tbody>
            {Object.values(NETWORKS).map((network) => (<tr key={network.id}>
                <td>{network.name}</td>
                <td>{network.id}</td>
            </tr>))}
            </tbody>
        </Table>

        <h6>🎯 Pagination and Filtering</h6>
        <p>
            The <code>GET {urlInvoices}</code> endpoint supports the following query
            parameters:
        </p>
        <ul>
            <li>
                <strong>limit:</strong> The maximum number of invoices to retrieve
                (default: 10).
            </li>
            <li>
                <strong>offset:</strong> The number of invoices to skip before starting
                retrieval (default: 0).
            </li>
            <li>
                <strong>user_id:</strong> Filter invoices based on ownership. Use{" "}
                <code>all</code> for all invoices or <code>my</code> for only the
                authenticated user’s invoices (default: all).
            </li>
        </ul>

        <h6>🚦 Response Codes</h6>
        <p>Here are some common response codes and their meanings:</p>
        <ul>
            <li>
                <strong>200 OK:</strong> The request was successful.
            </li>
            <li>
                <strong>201 Created:</strong> The invoice was successfully created.
            </li>
            <li>
                <strong>404 Not Found:</strong> The requested invoice does not exist.
            </li>
            <li>
                <strong>401 Unauthorized:</strong> Invalid or missing API key.
            </li>
            <li>
                <strong>429 Too Many Requests:</strong> You’ve exceeded the API rate
                limit.
            </li>
        </ul>
    </>);

}

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

    const urlInvoices = apiUrl("/payment/invoice");
    const urlInvoicesInstance = apiUrl("/payment/invoice/:invoice_id");

    const sections = [{
        id: "overview", title: "Overview", content: (<>
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
        </>),
    }, {
        id: "limits", title: "Limits", content: (<>
            <h5>Invoice Creation Limits</h5>
            <p>
                To ensure smooth operations, there are limits on the number of
                invoices you can create:
            </p>
            <ul>
                <li>
                    <strong>Web Users:</strong> Up to 100 invoices per day.
                </li>
                <li>
                    <strong>API Users:</strong> Up to 1,000 invoices per day.
                </li>
            </ul>
            <p>
                If you need higher limits, please contact our support team to explore
                available options.
            </p>
        </>),
    }, {
        id: "creating-invoices", title: "Creating Invoices", content: (<>
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
        </>),
    }, {
        id: "notifications", title: "Notifications", content: (<>
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
        </>),
    }, {
        id: "payments", title: "Making Payments", content: (<>
            <h5>How to Pay an Invoice</h5>
            <ol>
                <li>Open the invoice link shared by the seller.</li>
                <li>
                    Connect your MetaMask wallet to the supported blockchain network
                    listed in the invoice.
                </li>
                <li>Click "Pay Invoice" and confirm the payment in MetaMask.</li>
                <li>
                    Wait for the transaction to be confirmed on the blockchain (1–5
                    minutes on average).
                </li>
                <li>
                    Payment status will update automatically. Notifications will be
                    sent to both the buyer and seller upon completion.
                </li>
            </ol>
            <p>
                <strong>Note:</strong> Avoid paying the same invoice twice, as the
                system processes payments reliably. Double-check before initiating a
                transaction.
            </p>
        </>),
    }, {
        id: "faq", title: "FAQ", content: (<>
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
                        Enable email notifications by verifying your email address in
                        "Settings." Link your Telegram account to activate bot
                        notifications.
                    </p>
                </li>
            </ul>
        </>),
    },

        {
            id: "api-endpoints", title: "API Endpoints", notReady: false, content: (<APIEndpoints
                urlInvoices={urlInvoices}
                urlInvoicesInstance={urlInvoicesInstance}
            />),
        },];

    return (<>
        <Row>
            <Col>
                <h2 className="m-3">Documentation</h2>
            </Col>
        </Row>
        <Row className="mt-2">
            {/* Sidebar */}
            <Sidebar sections={sections}/>

            {/* Content Area */}
            <Col className="mt-md-0 mt-3" md={9}>
                {sections.map((section) => (<Section key={section.id} section={section}/>))}
            </Col>
        </Row>
    </>);
};

export default Documentation;