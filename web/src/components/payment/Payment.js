import React, { useEffect, useState } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { Container, Spinner, Alert, Card, Button, Modal } from 'react-bootstrap';
import AmountDisplay from "../common/AmountDisplay";
import LocalDate from '../common/LocalDate';
import {apiUrl} from "../../utils";
import axios from "axios";

function Payment() {
    const { payment_id } = useParams();
    const navigate = useNavigate();
    const [payment, setPayment] = useState(null);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState(null);
    const [rechecking, setRechecking] = useState(false);
    const [modal, setModal] = useState({ show: false, success: false, message: '' });

    useEffect(() => {
        axios.get(apiUrl(`/buy/payment/${payment_id}`), { withCredentials: true })
            .then(response => setPayment(response.data))
            .catch(err => {
                if (err.response && err.response.status === 404) {
                    navigate('/not-found');
                } else {
                    console.error(err);
                    setError('Failed to fetch payment details');
                }
            })
            .finally(() => setLoading(false));
    }, [payment_id, navigate]);

    const recheck = () => {
        setRechecking(true);
        axios.post(apiUrl(`/buy/payment/${payment_id}/recheck`), {}, { withCredentials: true })
            .then(response => {
                setPayment(response.data);
                setModal({ show: true, success: true, message: 'Payment has been successfully processed!' });
            })
            .catch(err => {
                if (err.response) {
                    setModal({
                        show: true,
                        success: false,
                        message: err.response.status === 400 ? `Message: ${err.response.data.message}` : 'Payment does not exist.',
                    });
                } else {
                    console.error(err);
                    setError('Failed to fetch payment details');
                }
            })
            .finally(() => setRechecking(false));
    };

    if (loading) {
        return (
            <Container className="mt-5 text-center">
                <Spinner animation="border" variant="primary" />
            </Container>
        );
    }

    if (error) {
        return (
            <Container className="mt-5 text-center">
                <Alert variant="danger">{error}</Alert>
            </Container>
        );
    }

    return (
        <Container className="mt-5" style={{ maxWidth: '600px' }}>
            <h2 className="mb-4">Payment Confirmation</h2>
            <PaymentInfo payment={payment} onRecheck={recheck} rechecking={rechecking} />
            <Modal show={modal.show} onHide={() => setModal({ ...modal, show: false })} centered>
                <Modal.Header closeButton>
                    <Modal.Title>{modal.success ? 'Success' : 'Error'}</Modal.Title>
                </Modal.Header>
                <Modal.Body>{modal.message}</Modal.Body>
                <Modal.Footer>
                    <Button variant="secondary" onClick={() => setModal({ ...modal, show: false })}>
                        Close
                    </Button>
                </Modal.Footer>
            </Modal>
        </Container>
    );
}

const paymentTemplates = {
    donation: ({payment, onRecheck, rechecking}) => (
        <Card className="p-3 bg-light text-dark">
            <Card.Title>{payment.paid_at ? "Thank You for Your Donation!" : "Donation Pending"}</Card.Title>
            <Card.Body>
                <p><strong>Amount:</strong> <AmountDisplay amount={payment.data.donation.amount} /></p>
                <p><strong>Donor:</strong> {payment.data.donation.donor || 'Anonymous'}</p>
                <p><strong>Date:</strong> <LocalDate date={payment.created_at} /></p>
                {payment.paid_at ? <Alert variant="success">Paid on <LocalDate date={payment.paid_at} /></Alert> : <Alert variant="warning">Payment pending</Alert>}
                {!payment.paid_at && <Button variant="dark" onClick={onRecheck} disabled={rechecking}>{rechecking ? 'Checking...' : 'Recheck Payment'}</Button>}
            </Card.Body>
        </Card>
    ),
    default: ({payment, onRecheck, rechecking}) => (
        <Card className="p-3 bg-light text-dark">
            <Card.Title>{payment.paid_at ? "Payment Processed" : "Payment Pending"}</Card.Title>
            <Card.Body>
                <p><strong>Created At:</strong> <LocalDate date={payment.created_at} /></p>
                {payment.paid_at ? (
                    <Alert variant="success">Paid on <LocalDate date={payment.paid_at} /></Alert>
                ) : (
                    <Alert variant="warning">Payment pending</Alert>
                )}
                {!payment.paid_at && <Button variant="dark" onClick={onRecheck} disabled={rechecking}>{rechecking ? 'Checking...' : 'Recheck Payment'}</Button>}
            </Card.Body>
        </Card>
    )
};

function PaymentInfo({ payment, onRecheck, rechecking }) {
    const paymentType = Object.keys(payment.data)[0];
    const PaymentComponent = paymentTemplates[paymentType] || paymentTemplates.default;
    return <PaymentComponent payment={payment} onRecheck={onRecheck} rechecking={rechecking} />;
}

export default Payment;
