import React, {useState} from 'react';
import {Container, Form, Button, Col, Row, Alert} from 'react-bootstrap';
import {getContacts, getSendMessageUrl} from "../utils";
import axios from "axios";

const Contact = () => {
    const [formData, setFormData] = useState({
        name: '',
        email: '',
        message: '',
        telegram: '',
    });
    const [alert, setAlert] = useState(null);

    const handleChange = (e) => {
        const {name, value} = e.target;
        setFormData({
            ...formData,
            [name]: value,
        });
    };

    const handleSubmit = async (e) => {
        e.preventDefault();

        const {name, email, message, telegram} = formData;
        const data = {
            contacts: {
                email,
                telegram,
                name,
            },
            text: message,
        };

        try {
            await axios.post(send_message_url, data, {headers: {'Content-Type': 'application/json'}});

            setAlert({
                type: 'success',
                message: 'Thank you for reaching out! We will get back to you shortly.',
            });

            setFormData({
                name: '',
                email: '',
                message: '',
                telegram: '',
            });
        } catch (error) {
            setAlert({
                type: 'danger',
                message: 'Oops! Something went wrong. Please try again.',
            });
        }
    };

    const send_message_url = getSendMessageUrl();
    const contacts = getContacts();
    const contact_email = contacts.email;
    const contact_telegram = contacts.telegram;
    const contact_linkedin = contacts.linkedin;

    return (
        <Container>
            <h2>Contact Us</h2>
            <p>If you have any questions or need support, please reach out to us using the form below.</p>

            {/* Display success or error messages */}
            {alert && (
                <Alert variant={alert.type}>
                    {alert.message}
                </Alert>
            )}

            <Form onSubmit={handleSubmit}>
                <Row>
                    <Col sm={12} md={6}>
                        <Form.Group controlId="formName">
                            <Form.Label>Name</Form.Label>
                            <Form.Control
                                type="text"
                                name="name"
                                value={formData.name}
                                onChange={handleChange}
                                required
                            />
                        </Form.Group>
                    </Col>
                    <Col sm={12} md={6}>
                        <Form.Group controlId="formEmail">
                            <Form.Label>Email</Form.Label>
                            <Form.Control
                                type="email"
                                name="email"
                                value={formData.email}
                                onChange={handleChange}
                                required
                            />
                        </Form.Group>
                    </Col>
                </Row>

                <Form.Group controlId="formMessage">
                    <Form.Label>Message</Form.Label>
                    <Form.Control
                        as="textarea"
                        name="message"
                        rows={4}
                        value={formData.message}
                        onChange={handleChange}
                        required
                    />
                </Form.Group>

                <Form.Group controlId="formTelegram">
                    <Form.Label>Telegram (optional)</Form.Label>
                    <Form.Control
                        type="text"
                        name="telegram"
                        value={formData.telegram}
                        onChange={handleChange}
                        placeholder="Enter your Telegram username (optional)"
                    />
                </Form.Group>

                <Button variant="dark" type="submit" className="mt-3">
                    Send Message
                </Button>
            </Form>

            <hr/>

            <h3>Other Ways to Reach Us</h3>
            <p className="m-3">Email <br/><code>{contact_email}</code></p>
            <p>If you'd like to get in touch directly, you can contact us through the following channels:</p>
            <div className="d-flex flex-wrap">
                <Button className="mx-3 my-1" variant="outline-dark" href={`https://t.me/${contact_telegram}`} target="_blank"
                        rel="noopener noreferrer">Telegram</Button>
                <Button className="mx-3 my-1" variant="outline-dark" href={`https://www.linkedin.com/in/${contact_linkedin}`}
                        target="_blank" rel="noopener noreferrer">LinkedIn</Button>
            </div>
        </Container>
    );
};

export default Contact;
