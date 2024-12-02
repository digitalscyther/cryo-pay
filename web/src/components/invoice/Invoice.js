import React, {useEffect, useState} from 'react';
import {useParams, useNavigate} from 'react-router-dom';
import axios from 'axios';
import {Alert, Container, Spinner} from 'react-bootstrap';
import {apiUrl, getBlockchainInfo} from "../../utils";
import Info from "./Info";
import Controller from "./Controller";

function Invoice() {
    const navigate = useNavigate();
    const {invoice_id} = useParams();
    const [invoice, setInvoice] = useState(null);
    const [own, setOwn] = useState(false);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState(null);
    const [erc20Abi, setErc20Abi] = useState(null);
    const [contractAbi, setContractAbi] = useState(null);
    const [networks, setNetworks] = useState(null);

    useEffect(() => {
        const fetchInvoice = async () => {
            try {
                const response = await axios.get(
                    apiUrl(`/payment/invoice/${invoice_id}`),
                    {withCredentials: true}
                );
                setInvoice(response.data.invoice);
                setOwn(response.data.own);
            } catch (err) {
                if (err.response && err.response.status === 404) {
                    navigate('/not-found');
                } else {
                    setError('Failed to fetch invoice data');
                }
            } finally {
                setLoading(false);
            }
        };

        const fetchBlockchainInfo = async () => {
            try {
                const response = await getBlockchainInfo();
                const {networks, abi} = response.data;

                setErc20Abi(abi.erc20);
                setContractAbi(abi.contract);
                setNetworks(networks.reduce((acc, item) => {
                    acc[item.id] = item;
                    return acc;
                }, {}));
            } catch (err) {
                setError('Failed to fetch blockchain info or connect to MetaMask');
            }
        };

        fetchInvoice();
        fetchBlockchainInfo();
    }, [invoice_id, navigate]);

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
            />}

            {error && (
                <Alert variant="danger" className="text-center mt-4">
                    {error}
                </Alert>
            )}
        </Container>
    );
}

export default Invoice;
