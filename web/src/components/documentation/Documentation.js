import React, {useEffect} from "react";
import {Row, Col} from "react-bootstrap";
import ApiEndpoints from "./ApiEndpoints";
import Section from "./Section";
import Sidebar from "./Sidebar";
import {apiUrl} from "../../utils";

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
                    Enable email notifications by verifying your email address in
                    "Settings." Link your Telegram account to activate bot
                    notifications.
                </p>
            </li>
        </ul>
    </>
)

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

    const sections = [
        {id: "overview", title: "Overview", content: (<OverviewContent/>)},
        {id: "limits", title: "Limits", content: (<LimitsContent/>)},
        {id: "creating-invoices", title: "Creating Invoices", content: (<CreateInvoiceContent/>)},
        {id: "payments", title: "Making Payments", content: (<MakingPaymentsContent/>)},
        {id: "notifications", title: "Notifications", content: (<NotificationsContent/>)},
        {id: "faq", title: "FAQ", content: (<FaqContent/>)},
        {
            id: "api-endpoints",
            title: "API Endpoints",
            content: (<ApiEndpoints urlInvoices={urlInvoices} urlInvoicesInstance={urlInvoicesInstance}/>),
        }
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