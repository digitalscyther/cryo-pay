import React, {useEffect, useState} from "react";
import {Container, Row, Col, Button, Card, ListGroup} from "react-bootstrap";
import NetworkIcon from "./NetworkIcon";
import ThumbnailWithZoom from "./ThumbnailWithZoom";
import CreateInvoice from "./dashboard/CreateInvoice";
import {getAvailableNetworks, getProjectName} from "../utils";
import {Link} from "react-router-dom";

const projectName = getProjectName();

const chooseReasons = [
    {
        title: "💎 Instant Crypto Payments",
        text: "Get paid directly with no middlemen."
    },
    {
        title: "🌐 Multi-Chain Support",
        text: "Accept payments via Optimism, Arbitrum, and more."
    },
    {
        title: "💻 Free API for Developers",
        text: "Build your integrations without limits."
    },
    {
        title: "🛡️ No Fees on Invoices",
        text: "Pay only gas fees—no platform charges."
    },
    {
        title: "📲 Real-Time Notifications",
        text: "Get instant payment alerts via Telegram or email."
    }
];

const reviews = [
    {
        text: `As a freelancer, ${projectName} has simplified my invoicing process. Fast and reliable!`,
        author: "Alex R., Freelancer"
    },
    {
        text: "Our business loves the multi-chain support and zero fees!",
        author: "Jamie L., Business Owner"
    },
    {
        text: "The real-time notifications and easy API integration are game-changers for our operations.",
        author: "Taylor M., Tech Lead"
    },
    {
        text: `No account needed, no hidden fees. ${projectName} is perfect for quick payments.`,
        author: "Jordan S., Consultant"
    }
];

const schemas = [
    {
        src: "/files/payment-scheme.svg",
        title: (
            <>
                Payment Process:<br/>From Creation to Notification
            </>
        ),
        description: "Visual representation of how payments are processed and notifications sent."
    },
    {
        src: "/files/buy-scheme.svg",
        title: "How It Works on the Blockchain",
        description: "Understanding the decentralized nature of transactions with no middlemen."
    },
]

const steps = [
    "1️⃣ Enter amount and invoice details — it takes seconds.",
    "2️⃣ Share your unique payment link with your client instantly.",
    "3️⃣ Receive crypto payments directly, with real-time notifications."
]

const Home = () => {
    const [availableNetworks, setAvailableNetworks] = useState([]);

    useEffect(() => {
        getAvailableNetworks().then(setAvailableNetworks);
    }, []);

    return (
        <Container fluid>
            {/* Hero Section */}
            <div className="bg-dark text-white text-center py-5">
                <Row>
                    <Col>
                        <h1 className="display-5">Create Crypto Invoices Instantly</h1>
                        <h2 className="display-3 p-3">No Fees, No Sign-Up, Just a Link</h2>
                        <p className="lead">Fast, secure, and free invoicing with decentralized networks.</p>
                        <Row className="justify-content-center mb-4">
                            {availableNetworks.map(network => (
                                <Col key={network} xs="auto" className="mx-2">
                                    <NetworkIcon size={40} networkName={network}/>
                                </Col>
                            ))}
                        </Row>
                        <div className="mb-4">
                            <CreateInvoice
                                createBtn={(onClick) => (
                                    <Button className="m-2" variant="light" size="lg" onClick={onClick}>
                                        🚀 Create Invoice – It’s Free
                                    </Button>
                                )}
                                showExternalBlock={false}
                                promoFeature={(
                                    <p className="mb-0">
                                        Want more features?
                                        <br/>
                                        <Link to="/dashboard">Explore your Dashboard with all your invoices →</Link>
                                    </p>
                                )}
                            />
                            <Button className="m-2" variant="outline-light" size="lg" href="#how-it-works">
                                🔍 Watch Demo
                            </Button>
                        </div>
                    </Col>
                </Row>
            </div>

            <section className="my-5 text-center">
                <h2>Explore how our platform works</h2>
                <Row className="justify-content-center d-flex">
                    {schemas.map(({title, description, src}) => (
                        <Col key={title} md={4} sm={6} xs={12} className="mb-4 d-flex">
                            <Card className="shadow-lg border-0 rounded-2 d-flex flex-column h-100">
                                <Card.Body className="d-flex flex-column">
                                    <Card.Title className="fs-5">{title}</Card.Title>
                                    <Card.Text className="text-muted small flex-grow-1 d-flex flex-column">
                                        <p>{description}</p>
                                        <div className="m-auto">
                                            <ThumbnailWithZoom
                                                src={src}
                                                altText={title}
                                                thumbnailWidthSize="250px"
                                            />
                                        </div>
                                    </Card.Text>
                                </Card.Body>
                            </Card>
                        </Col>
                    ))}
                </Row>
            </section>


            {/* How It Works Section */}
            <section id="how-it-works" className="my-5 text-center">
                <h2>See How It Works (30 seconds)</h2>
                <div className="d-flex justify-content-center my-4">
                    <iframe
                        width="560" height="315"
                        src="https://www.youtube.com/embed/F7jpBl3cYXM"
                        title="Demo video"
                        frameBorder="0"
                        allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture"
                        allowFullScreen
                    />
                </div>
                <p><i>Create and share your invoice in under 30 seconds — it's that easy!</i></p>
                <h4>Create Your First Invoice in 3 Steps:</h4>
                <ListGroup className="mx-auto my-3 d-flex text-start">
                    <div className="m-auto">
                        {steps.map((text) => (
                            <ListGroup.Item>{text}</ListGroup.Item>
                        ))}
                    </div>
                </ListGroup>
            </section>

            {/* Why Choose Section */}
            <section className="my-5 text-center">
                <h2>Why Choose {projectName}?</h2>
                <Row className="justify-content-center">
                    {chooseReasons.map(({title, text}) => (
                        <Col key={title} md={3} className="mb-3">
                            <Card>
                                <Card.Body>
                                    <Card.Title>{title}</Card.Title>
                                    <Card.Text>{text}</Card.Text>
                                </Card.Body>
                            </Card>
                        </Col>
                    ))}
                </Row>
            </section>

            {/* Reviews Section */}
            <section className="my-5 text-center">
                <h2>What Our Users Say</h2>
                <Row className="justify-content-center">
                    {reviews.map(({text, author}) => (
                        <Col key={author} md={4} className="mb-3">
                            <Card>
                                <Card.Body>
                                    <Card.Text>"{text}"</Card.Text>
                                    <Card.Footer className="text-end">- {author}</Card.Footer>
                                </Card.Body>
                            </Card>
                        </Col>
                    ))}
                </Row>
            </section>

            {/* Join Section - Enhanced */}
            <section className="my-5 text-center bg-light p-4 rounded shadow">
                <h2 className="fw-bold">🚀 Be Part of Something Big — Over 100 Invoices Created!</h2>
                <p className="mt-2 fs-5 text-muted">Join a growing community simplifying payments and invoicing
                    effortlessly.</p>
                <Button className="m-2" variant="dark" size="lg" href="/dashboard">
                    Get Started for Free
                </Button>
            </section>

            {/* Developer Section */}
            <section className="my-5 text-center">
                <h3>💻 For Developers</h3>
                <p>
                    Integrate <Link to="/docs#api-endpoints">our API</Link> and start accepting crypto payments
                    today.
                    Enjoy a free plan with up to 10 invoices/day.
                </p>
            </section>
        </Container>
    );
};

export default Home;

