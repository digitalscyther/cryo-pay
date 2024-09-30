import React, {useEffect, useState} from 'react';
import {useParams} from 'react-router-dom';
import axios from 'axios';
import Web3 from 'web3';
import {Container, Button, Spinner, Alert} from 'react-bootstrap';

const ERC20_ADDRESS = "0x9A211fD6C60BdC4Cc1dB22cBe2f882ae527B1D87";  // Hardcoded ERC20 token address
const INVOICE_CONTRACT_ADDRESS = "0xb9BB9B797a90bf2aA212C92E4d100F39cD8E325c";  // Hardcoded smart contract address
const INVOICE_ABI = [
    {
        "inputs": [
            {
                "internalType": "address",
                "name": "seller",
                "type": "address"
            },
            {
                "internalType": "string",
                "name": "invoice_id",
                "type": "string"
            },
            {
                "internalType": "uint256",
                "name": "amount",
                "type": "uint256"
            }
        ],
        "name": "payInvoice",
        "outputs": [],
        "stateMutability": "nonpayable",
        "type": "function"
    }
];
const ERC20_ABI = [
    {
        "constant": false,
        "inputs": [
            {
                "name": "_spender",
                "type": "address"
            },
            {
                "name": "_value",
                "type": "uint256"
            }
        ],
        "name": "approve",
        "outputs": [
            {
                "name": "",
                "type": "bool"
            }
        ],
        "payable": false,
        "stateMutability": "nonpayable",
        "type": "function"
    }
];

function PayInvoice() {
    const {invoice_id} = useParams(); // Get invoice_id from the route
    const [invoice, setInvoice] = useState(null);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState(null);
    const [processing, setProcessing] = useState(false);
    const [web3, setWeb3] = useState(null);
    const [account, setAccount] = useState(null);

    useEffect(() => {
        // Fetch the invoice details
        axios
            .get(`http://localhost:3000/payment/invoice/${invoice_id}`)
            .then((response) => {
                setInvoice(response.data);
                setLoading(false);
            })
            .catch((err) => {
                setError('Failed to fetch invoice data');
                setLoading(false);
            });

        // Initialize Web3 and MetaMask
        if (window.ethereum) {
            const web3Instance = new Web3(window.ethereum);
            setWeb3(web3Instance);
            window.ethereum.request({method: 'eth_requestAccounts'})
                .then((accounts) => setAccount(accounts[0]))
                .catch((err) => setError('MetaMask connection failed'));
        } else {
            setError('Please install MetaMask!');
        }
    }, [invoice_id]);

    const handlePayment = async () => {
        if (!web3 || !account || !invoice) return;

        try {
            setProcessing(true);

            // 1. Approve the contract to spend the user's tokens
            const erc20Contract = new web3.eth.Contract(ERC20_ABI, ERC20_ADDRESS);
            const invoiceContract = new web3.eth.Contract(INVOICE_ABI, INVOICE_CONTRACT_ADDRESS);
            const amount = invoice.amount * (10**6); // Convert the amount

            // Approve tokens for the InvoicePayment contract
            await erc20Contract.methods.approve(INVOICE_CONTRACT_ADDRESS, amount).send({from: account});
            console.log('Approval successful');

            // 2. Make the transaction to pay the invoice
            await invoiceContract.methods.payInvoice(invoice.seller, invoice_id, amount).send({from: account});
            console.log('Payment successful');

            // 3. Show success message
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

    if (error) {
        return (
            <Container className="mt-5">
                <Alert variant="danger">{error}</Alert>
            </Container>
        );
    }

    if (invoice.paid_at) {
        return (
            <Container className="mt-5">
                <Alert variant="success">This invoice has already been paid.</Alert>
            </Container>
        );
    }

    return (
        <Container className="mt-5">
            <h2>Pay Invoice</h2>
            <p><strong>Invoice ID:</strong> {invoice.id}</p>
            <p><strong>Amount:</strong> {parseFloat(invoice.amount).toFixed(2)} MTK</p>
            <p><strong>Seller:</strong> {invoice.seller}</p>
            <p><strong>Created At:</strong> {new Date(invoice.created_at).toLocaleString()}</p>

            {processing ? (
                <Button variant="primary" disabled>
                    Processing Payment...
                </Button>
            ) : (
                <Button variant="primary" onClick={handlePayment}>
                    Pay with MetaMask
                </Button>
            )}
        </Container>
    );
}

export default PayInvoice;
