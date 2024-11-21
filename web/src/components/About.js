import React from 'react';
import { Container, Row, Col, Card, ListGroup, Button, Accordion } from 'react-bootstrap';
import {getProjectGitHubUrl, getProjectName} from "../utils";

const About = () => {
    const projectGitHubUrl = getProjectGitHubUrl();
    const projectName = getProjectName();
    const commission = "0–1%";

    return (
        <Container className="mt-5">
            <Row className="text-center">
                <Col>
                    <h1 className="display-4">Welcome to { projectName }</h1>
                    <p className="lead">
                        { projectName } offers a cutting-edge platform for seamless invoice management,
                        multi-network blockchain integration, and secure transactions designed for modern businesses.
                    </p>
                </Col>
            </Row>

            <Row className="mt-5">
                <Col md={6}>
                    <Card className="shadow-sm">
                        <Card.Header as="h5" className="bg-dark text-white text-center">
                            Why Choose { projectName }
                        </Card.Header>
                        <Card.Body>
                            <Card.Text>
                                Our platform simplifies billing for businesses by enabling secure, efficient, and blockchain-powered
                                invoice management. With multi-network support and tailored solutions, { projectName } ensures
                                transparency, scalability, and user-friendly features.
                            </Card.Text>
                        </Card.Body>
                    </Card>
                </Col>
                <Col md={6}>
                    <Card className="shadow-sm">
                        <Card.Header as="h5" className="bg-dark text-white text-center">
                            Key Features
                        </Card.Header>
                        <Card.Body>
                            <ListGroup variant="flush">
                                <ListGroup.Item>Blockchain integration: Optimism, Arbitrum, and more.</ListGroup.Item>
                                <ListGroup.Item>Automatic commission retention of { commission }.</ListGroup.Item>
                                <ListGroup.Item>Real-time notifications via email and Telegram.</ListGroup.Item>
                                <ListGroup.Item>Support for ERC20 tokens and smart contract invoicing.</ListGroup.Item>
                                <ListGroup.Item>Secure OAuth2 and JWT-based authorization.</ListGroup.Item>
                            </ListGroup>
                        </Card.Body>
                    </Card>
                </Col>
            </Row>

            <Row className="mt-5">
                <Col md={4}>
                    <Card className="shadow-sm">
                        <Card.Body>
                            <Card.Title>Multi-Network Support</Card.Title>
                            <Card.Text>
                                Our platform operates seamlessly across multiple blockchain networks such as Optimism,
                                Arbitrum, and more, ensuring flexibility for diverse user needs.
                            </Card.Text>
                        </Card.Body>
                    </Card>
                </Col>
                <Col md={4}>
                    <Card className="shadow-sm">
                        <Card.Body>
                            <Card.Title>Notifications</Card.Title>
                            <Card.Text>
                                Stay informed with real-time payment status updates delivered through email and Telegram
                                notifications, tailored to your preferences.
                            </Card.Text>
                        </Card.Body>
                    </Card>
                </Col>
                <Col md={4}>
                    <Card className="shadow-sm">
                        <Card.Body>
                            <Card.Title>Analytics and Reporting</Card.Title>
                            <Card.Text>
                                Access detailed statistics, transaction history, and reporting tools to monitor your
                                business performance and streamline operations.
                            </Card.Text>
                        </Card.Body>
                    </Card>
                </Col>
            </Row>

            <Row className="mt-5">
                <Col>
                    <h3 className="text-center">Frequently Asked Questions</h3>
                    <Accordion>
                        <Accordion.Item eventKey="0">
                            <Accordion.Header>Which networks are supported?</Accordion.Header>
                            <Accordion.Body>
                                We currently support multiple networks, including Optimism and Arbitrum, with plans to expand as new
                                networks gain traction.
                            </Accordion.Body>
                        </Accordion.Item>
                        <Accordion.Item eventKey="1">
                            <Accordion.Header>What is the commission system?</Accordion.Header>
                            <Accordion.Body>
                                Our smart contracts retain a small commission ({ commission }) on transactions for operational costs, ensuring transparency and efficiency.
                            </Accordion.Body>
                        </Accordion.Item>
                        <Accordion.Item eventKey="2">
                            <Accordion.Header>How are notifications sent?</Accordion.Header>
                            <Accordion.Body>
                                Notifications about payment statuses can be sent via email and Telegram, depending on your configured preferences.
                            </Accordion.Body>
                        </Accordion.Item>
                        <Accordion.Item eventKey="3">
                            <Accordion.Header>What’s next for { projectName }?</Accordion.Header>
                            <Accordion.Body>
                                Upcoming features include bulk invoice creation, contact information storage, QR code generation,
                                advanced statistics, and more.
                            </Accordion.Body>
                        </Accordion.Item>
                    </Accordion>
                </Col>
            </Row>

            <Row className="mt-5 text-center">
                <Col>
                    <h3>Additional Resources</h3>
                    <div className="d-flex justify-content-center">
                        <div>
                            <a href={projectGitHubUrl} target="_blank" rel="noopener noreferrer" className="m-2">
                                <Button variant="secondary">
                                    GitHub Repository
                                </Button>
                            </a>
                        </div>
                        <div>
                            <a href="/docs" target="_blank" rel="noopener noreferrer" className="m-2">
                                <Button variant="secondary">
                                    Documentation
                                </Button>
                            </a>
                        </div>
                    </div>
                </Col>
            </Row>

            <Row className="m-4 text-center">
                <Col>
                    <Button className="btn-outline-dark" variant="light" href="/contact" size="lg">
                        Contact Us
                    </Button>
                </Col>
            </Row>
        </Container>
    );
};

export default About;
