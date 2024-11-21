import React from "react";
import { Container, Row, Col, Button, Card } from "react-bootstrap";
import {getProjectName} from "../utils";

const Home = ({ isLoggedIn }) => {
    const projectName = getProjectName();

    return (
        <>
            {/* Hero Section */}
            <Container fluid className="bg-dark text-white text-center py-5">
                <Row>
                    <Col>
                        <h1 className="display-4">Simplifying Payments for Sellers with Blockchain</h1>
                        <p className="lead">
                            Fast, secure, and low-cost invoicing powered by decentralized networks.
                        </p>
                        <div className="mt-4">
                            <Button className="m-2" variant="primary" size="lg" href="/dashboard">
                                {isLoggedIn ? "Go to Dashboard" : "Get Started"}
                            </Button>
                            <Button className="m-2" variant="outline-light" size="lg" href="/about">
                                Learn More
                            </Button>
                        </div>
                    </Col>
                </Row>
            </Container>

            {/* Features Section */}
            <Container className="mt-5">
                <h2 className="text-center">Why Choose { projectName }?</h2>
                <Row className="mt-4">
                    <Col md={4} className="mb-3">
                        <Card>
                            <Card.Body>
                                <Card.Title>Multiple Network Support</Card.Title>
                                <Card.Text>
                                    Seamlessly connect to Optimism, Arbitrum, and other blockchain networks.
                                </Card.Text>
                            </Card.Body>
                        </Card>
                    </Col>
                    <Col md={4} className="mb-3">
                        <Card>
                            <Card.Body>
                                <Card.Title>Low Gas Fees</Card.Title>
                                <Card.Text>
                                    Our platform ensures minimal transaction costs with competitive gas prices.
                                </Card.Text>
                            </Card.Body>
                        </Card>
                    </Col>
                    <Col md={4} className="mb-3">
                        <Card>
                            <Card.Body>
                                <Card.Title>Real-Time Notifications</Card.Title>
                                <Card.Text>
                                    Stay updated with instant notifications via email or Telegram.
                                </Card.Text>
                            </Card.Body>
                        </Card>
                    </Col>
                </Row>
            </Container>

            {/* Testimonials Section */}
            <Container className="my-5">
                <h2 className="text-center">What Our Users Say</h2>
                <Row className="mt-4">
                    <Col md={6} className="mb-3">
                        <Card>
                            <Card.Body>
                                <Card.Text>
                                    "{ projectName } has streamlined my invoicing process. The low gas fees are a game-changer!"
                                </Card.Text>
                                <Card.Footer className="text-end">- Alex R.</Card.Footer>
                            </Card.Body>
                        </Card>
                    </Col>
                    <Col md={6} className="mb-3">
                        <Card>
                            <Card.Body>
                                <Card.Text>
                                    "I love the multi-network support. Switching between chains has never been easier!"
                                </Card.Text>
                                <Card.Footer className="text-end">- Jamie L.</Card.Footer>
                            </Card.Body>
                        </Card>
                    </Col>
                </Row>
            </Container>

            {/* Call-to-Action Section */}
            <Container fluid className="bg-light text-center py-5">
                <h2>Ready to Simplify Your Billing?</h2>
                <p className="lead">Create invoices effortlessly and manage payments with ease.</p>
                <div>
                    <Button className="m-2" variant="success" size="lg" href="/dashboard">
                        Create an Invoice
                    </Button>
                    <Button className="m-2" variant="outline-dark" size="lg" href="/about">
                        Learn More
                    </Button>
                </div>
            </Container>
        </>
    );
};

export default Home;
