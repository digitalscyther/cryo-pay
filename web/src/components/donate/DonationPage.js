import React, { useState } from 'react';
import DonationList from './DonationList';
import DonationModal from './DonationModal';
import { Alert, Button, Col, Container, Row } from 'react-bootstrap';
import axios from 'axios';
import { apiUrl } from '../../utils';

function DonationPage() {
    const [showModal, setShowModal] = useState(false);
    const [error, setError] = useState(null);

    const handleDonateInitiate = async (amount) => {
        setError(null);
        try {
            const response = await axios.post(apiUrl('/buy/donation'), { amount }, { withCredentials: true });
            window.location.href = response.data.payment_url;
        } catch (err) {
            if (err.response?.status === 429) {
                setError('You have reached the limit of creating invoices for today. Please try again tomorrow.');
            } else {
                setError('An error occurred while processing your donation. Please try again.');
                console.error('Failed to create invoice:', err);
            }
        }
    };

    return (
        <Container className="mt-5">
            <Row className="justify-content-center">
                <Col md={8} className="text-center">
                    <h2 className="fw-bold">Support Our Cause</h2>
                    <p className="text-muted">Your donations help us continue our mission. Every contribution makes a difference.</p>
                    <Button variant="dark" onClick={() => setShowModal(true)} className="px-4 py-2">
                        Donate Now
                    </Button>
                    {error && <Alert variant="danger" className="mt-3">{error}</Alert>}
                </Col>
            </Row>
            <Row className="mt-5">
                <Col>
                    <h3 className="fw-semibold">Recent Donations</h3>
                    <DonationList />
                </Col>
            </Row>
            <DonationModal show={showModal} onHide={() => setShowModal(false)} onDonate={handleDonateInitiate} />
        </Container>
    );
}

export default DonationPage;
