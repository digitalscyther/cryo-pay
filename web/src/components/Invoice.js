import React, {useEffect, useState} from 'react';
import {useParams} from 'react-router-dom';
import axios from 'axios';
import Web3 from 'web3';
import {Container, Button, Spinner, Alert} from 'react-bootstrap';
import {api_url, SEPOLIA_OPTIMISM_NETWORK_ID} from "../utils";

function Invoice() {
    const {invoice_id} = useParams();
    const [invoice, setInvoice] = useState(null);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState(null);
    const [processing, setProcessing] = useState(false);
    const [web3, setWeb3] = useState(null);
    const [account, setAccount] = useState(null);
    const [erc20Contract, setErc20Contract] = useState(null);
    const [invoiceContract, setInvoiceContract] = useState(null);

    useEffect(() => {
        const fetchInvoice = async () => {
            try {
                const response = await axios.get(api_url(`/payment/invoice/${invoice_id}`));
                setInvoice(response.data);
            } catch (err) {
                setError('Failed to fetch invoice data');
            } finally {
                setLoading(false);
            }
        };

        const fetchBlockchainInfo = async () => {
            try {
                const response = await axios.get(api_url('/blockchain/info'));
                const {erc20, invoice} = response.data;

                const web3Instance = new Web3(window.ethereum);
                const erc20ContractInstance = new web3Instance.eth.Contract(erc20.abi, erc20.address);
                const invoiceContractInstance = new web3Instance.eth.Contract(invoice.abi, invoice.address);

                setErc20Contract(erc20ContractInstance);
                setInvoiceContract(invoiceContractInstance);
                setWeb3(web3Instance);

                const accounts = await window.ethereum.request({method: 'eth_requestAccounts'});
                setAccount(accounts[0]);
            } catch (err) {
                setError('Failed to fetch blockchain info or connect to MetaMask');
            }
        };

        fetchInvoice();
        fetchBlockchainInfo();
    }, [invoice_id]);

    const handlePayment = async () => {
        const isValidState = () => web3 && account && invoice && erc20Contract && invoiceContract;
        const getNetworkId = () => web3.eth.net.getId();
        const handleApproval = (amount) => () => erc20Contract.methods.approve(invoiceContract._address, amount).send({from: account});
        const handlePaymentTransaction = (amount) => () => invoiceContract.methods.payInvoice(invoice.seller, invoice_id, amount).send({from: account});

        setError(null);
        if (!isValidState()) return;

        const processPayment = async (amount) =>
            handleApproval(amount)()
                .then(() => console.log('Approval successful'))
                .then(handlePaymentTransaction(amount))
                .then(() => console.log('Payment successful'));

        try {
            const networkId = await getNetworkId();

            if (networkId !== SEPOLIA_OPTIMISM_NETWORK_ID) {
                return setError('Please switch to the correct network');
            }

            setProcessing(true);
            const amount = invoice.amount * (10 ** 6);

            await processPayment(amount);
            alert('Payment is under processing. It will be marked as paid once everything is fine.');
        } catch (error) {
            console.error('Payment failed', error);
            setError('Payment failed, please try again');
        } finally {
            setProcessing(false);
        }
    };

    if (loading) {
        return (
            <Container className="mt-5 text-center">
                <Spinner animation="border" variant="primary"/>
            </Container>
        );
    }

    return (
        <Container className="mt-5">
            <h2>Pay Invoice</h2>
            <p><strong>Invoice ID:</strong> {invoice.id}</p>
            <p><strong>Amount:</strong> {parseFloat(invoice.amount).toFixed(2)} MTK</p>
            <p><strong>Network:</strong> Sepolia-Optimism (test)</p>
            <p><strong>Seller:</strong> {invoice.seller}</p>
            <p><strong>Created At:</strong> {new Date(invoice.created_at).toLocaleString()}</p>

            {invoice.paid_at ? (
                <>
                    <p><strong>Paid At:</strong> {new Date(invoice.paid_at).toLocaleString()}</p>
                    <Alert variant="success">This invoice has already been paid.</Alert>
                </>
            ) : (
                <>
                    {processing ? (
                        <Button variant="primary" disabled>
                            Processing Payment...
                        </Button>
                    ) : (
                        <Button variant="primary" onClick={handlePayment}>
                            Pay with MetaMask
                        </Button>
                    )}
                </>
            )}

            {error && <Alert className="my-3" variant="danger">{error}</Alert>}

        </Container>
    );
}

export default Invoice;
