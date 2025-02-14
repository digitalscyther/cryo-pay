import React, { useState } from 'react';
import { Modal, Button, Form } from 'react-bootstrap';

function DonationModal({ show, onHide, onDonate }) {
    const [donationAmount, setDonationAmount] = useState(null);
    const [customAmount, setCustomAmount] = useState('');
    const [isCustomAmount, setIsCustomAmount] = useState(false);

    const handleAmountSelect = (amount) => {
        setDonationAmount(amount);
        setCustomAmount('');
        setIsCustomAmount(false);
    };

    const handleCustomAmountChange = (event) => {
        setCustomAmount(event.target.value);
        setIsCustomAmount(true);
        setDonationAmount(null);
    };

    const handleDonateClick = () => {
        let amountToDonate = isCustomAmount ? customAmount : donationAmount;
        if (!amountToDonate || isNaN(amountToDonate) || parseFloat(amountToDonate) <= 0) {
            alert("Please enter a valid donation amount.");
            return;
        }
        onDonate(parseFloat(amountToDonate));
        onHide();
    };

    return (
        <Modal show={show} onHide={onHide} className="text-dark">
            <Modal.Header closeButton>
                <Modal.Title>Choose Donation Amount</Modal.Title>
            </Modal.Header>
            <Modal.Body className="bg-light text-dark">
                <p>Select a pre-defined amount or enter a custom amount.</p>
                <div className="d-flex flex-wrap gap-2">
                    {[1, 5, 10, 50, 100].map(amount => (
                        <Button
                            key={amount}
                            variant={donationAmount === amount ? "dark" : "outline-dark"}
                            onClick={() => handleAmountSelect(amount)}
                            className="px-3"
                        >
                            ${amount}
                        </Button>
                    ))}
                </div>
                <Form.Group className="mt-3">
                    <Form.Label>Custom Amount:</Form.Label>
                    <Form.Control
                        type="number"
                        placeholder="Enter custom amount"
                        value={customAmount}
                        onChange={handleCustomAmountChange}
                        className="border-dark"
                    />
                </Form.Group>
            </Modal.Body>
            <Modal.Footer>
                <Button variant="outline-dark" onClick={onHide}>
                    Close
                </Button>
                <Button variant="dark" onClick={handleDonateClick}>
                    Donate Now
                </Button>
            </Modal.Footer>
        </Modal>
    );
}

export default DonationModal;
