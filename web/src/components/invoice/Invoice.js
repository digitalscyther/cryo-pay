import React, {useEffect, useState} from 'react';
import {useParams, useNavigate, useSearchParams} from 'react-router-dom';
import {Alert, Container, Spinner, Modal, Button} from 'react-bootstrap';
import {apiUrl, getBlockchainInfo, getInvoice} from "../../utils";
import Info from "./Info";
import Controller from "./Controller";

const updateIfNotPaidAfterSeconds = 10;

function Invoice() {
    const navigate = useNavigate();
    const [searchParams] = useSearchParams();
    const {invoice_id} = useParams();
    const [invoice, setInvoice] = useState(null);
    const [own, setOwn] = useState(false);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState(null);
    const [erc20Abi, setErc20Abi] = useState(null);
    const [contractAbi, setContractAbi] = useState(null);
    const [networks, setNetworks] = useState(null);
    const [showModal, setShowModal] = useState(false);
    const [showWait, setShowWait] = useState(false);
    const callbackUrl = searchParams.get('callback_url');

    useEffect(() => {
        getBlockchainInfo()
            .then((response) => {
                const {networks, abi} = response.data;

                setErc20Abi(abi.erc20);
                setContractAbi(abi.contract);
                setNetworks(networks.reduce((acc, item) => {
                    acc[item.id] = item;
                    return acc;
                }, {}));
            })
            .catch((err) => {
                console.error(err);
                setError('Failed to fetch blockchain info or connect to MetaMask');
            });
    }, []);

    useEffect(() => {
        getInvoice(invoice_id)
            .then((response) => {
                setInvoice(response.data.invoice);
                setOwn(response.data.own);
            })
            .catch((err) => {
                if (err.response && err.response.status === 404) {
                    navigate('/not-found');
                } else {
                    console.error(err);
                    setError('Failed to fetch invoice data');
                }
            })
            .finally(() => setLoading(false));
    }, [invoice_id, navigate]);

    useEffect(() => {
        if (!!invoice && !invoice.paid_at) {
            const interval = setInterval(() => {
                getInvoice(invoice_id)
                    .then((response) => {
                        if (!!response.data.invoice.paid_at) {
                            if (callbackUrl) {
                                window.location.href = apiUrl(`/payment/invoice/${invoice_id}/redirect?url=${encodeURIComponent(callbackUrl)}`);
                            } else {
                                navigate(0);
                            }
                        }
                    })
                    .catch((err) => console.error("Failed to monitor invoice paid_at", err));
            }, updateIfNotPaidAfterSeconds * 1000);
            return () => clearInterval(interval);
        }
    }, [invoice_id, invoice, navigate, callbackUrl]);

    function waitPaymentSuccessful() {
        if (callbackUrl) {
            setShowWait(true);
        }
        setShowModal(true);
    }

    if (loading) {
        return (
            <Container className="mt-5 text-center">
                <Spinner animation="border" variant="primary"/>
            </Container>
        );
    }

    return (
        <Container className="mt-5" style={{maxWidth: '600px'}}>
            <h2 className="mb-4">Pay Invoice</h2>

            {showWait && (
                <Alert variant="danger" className="text-center">
                    Please wait while the transaction is being verified.
                    <br />
                    Do not leave the page or close your browser
                </Alert>
            )}

            <Info invoice={invoice}/>

            {invoice.paid_at ? (
                <Alert variant="success" className="text-center">
                    This invoice has already been paid.
                </Alert>
            ) : <Controller
                invoice={invoice}
                own={own}
                erc20Abi={erc20Abi}
                contractAbi={contractAbi}
                networks={networks}
                waitPaymentSuccessful={waitPaymentSuccessful}
            />}

            {error && (
                <Alert variant="danger" className="text-center mt-4">
                    {error}
                </Alert>
            )}

            <PaymentInProcess
                showModal={showModal}
                setShowModal={setShowModal}
            />
        </Container>
    );
}

function PaymentInProcess({showModal, setShowModal}) {
    return (
        <Modal show={showModal} onHide={() => setShowModal(false)}>
            <Modal.Header closeButton>
                <Modal.Title>Payment Processing</Modal.Title>
            </Modal.Header>
            <Modal.Body>
                Your payment is being processed. It will be marked as paid once confirmed.
            </Modal.Body>
            <Modal.Footer>
                <Button variant="secondary" onClick={() => setShowModal(false)}>
                    Close
                </Button>
            </Modal.Footer>
        </Modal>
    )
}

export default Invoice;
