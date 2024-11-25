import React, {useEffect, useState} from 'react';
import {Alert, Button, Spinner, Table, Form} from 'react-bootstrap';
import axios from 'axios';
import {useSearchParams} from 'react-router-dom';
import {apiUrl} from "../utils";
import AmountDisplay from "./AmountDisplay";
import LocalDate from './LocalDate';

const PAGE_SIZE = 10;

const InvoiceList = ({isLoggedIn}) => {
    const [invoices, setInvoices] = useState([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState(null);
    const [hasMore, setHasMore] = useState(false);

    const [searchParams, setSearchParams] = useSearchParams();

    const page = parseInt(searchParams.get('page') || '1', 10);
    const onlyOwner = searchParams.get('owner') === '1';

    const offset = (page - 1) * PAGE_SIZE;

    useEffect(() => {
        const fetchInvoices = () => {
            setLoading(true);
            const userFilter = onlyOwner ? "user_id=my&" : "";

            axios
                .get(
                    apiUrl(`/payment/invoice?${userFilter}limit=${PAGE_SIZE}&offset=${offset}`),
                    {withCredentials: true}
                )
                .then((response) => {
                    setInvoices(response.data);
                    setHasMore(response.data.length === PAGE_SIZE);
                    setLoading(false);
                })
                .catch((err) => {
                    console.log(`Failed to fetch invoices: ${err}`);
                    setError('Failed to fetch invoices');
                    setLoading(false);
                });
        };

        fetchInvoices();
    }, [offset, onlyOwner]);

    const handleOnlyOwnerChange = (e) => {
        const checked = e.target.checked;
        setSearchParams({
            ...(checked && {owner: '1'}),
        });
    };

    const goToPage = (newPage) => {
        setSearchParams({
            ...(newPage > 1 && {page: newPage}),
            ...(onlyOwner && {owner: '1'}),
        });
    };

    if (loading) {
        return (
            <div className="text-center mt-5">
                <Spinner animation="border" variant="primary"/>
            </div>
        );
    }

    if (error) {
        return (
            <Alert variant="danger" className="mt-5">
                {error}
            </Alert>
        );
    }

    return (
        <>
            <div className="d-flex justify-content-end mt-3 mb-1 me-3">
                {isLoggedIn && (
                    <Form className="me-auto">
                        <Form.Check
                            className="text-primary"
                            type="switch"
                            label="Only My"
                            checked={onlyOwner}
                            onChange={handleOnlyOwnerChange}
                        />
                    </Form>
                )}
                <Button
                    variant="primary"
                    disabled={page === 1}
                    onClick={() => goToPage(page - 1)}
                >
                    &laquo; {/* Previous */}
                </Button>
                <div className="mx-3 my-auto text-primary" style={{ fontSize: "1.2em"}}>{page}</div>
                <Button
                    variant="primary"
                    disabled={!hasMore}
                    onClick={() => goToPage(page + 1)}
                >
                    &raquo; {/* Next */}
                </Button>
            </div>

            {invoices.length === 0 ? (
                <Alert variant="info">No invoices found</Alert>
            ) : (
                <Table striped bordered hover responsive>
                    <thead>
                    <tr>
                        <th>ID</th>
                        <th>Amount</th>
                        <th>Created At</th>
                        <th>Paid At</th>
                    </tr>
                    </thead>
                    <tbody>
                    {invoices.map((invoice) => (
                        <tr key={invoice.id}>
                            <td>
                                <Button
                                    variant="link"
                                    onClick={() =>
                                        (window.location.href = `/invoices/${invoice.id}`)
                                    }
                                >
                                    {invoice.id}
                                </Button>
                            </td>
                            <td>
                                <AmountDisplay amount={invoice.amount} size={1.0}/>
                            </td>
                            <td>
                                <LocalDate date={invoice.created_at}/>
                            </td>
                            <td>
                                {invoice.paid_at ? (
                                    <LocalDate date={invoice.paid_at}/>
                                ) : (
                                    'Unpaid'
                                )}
                            </td>
                        </tr>
                    ))}
                    </tbody>
                </Table>
            )}
        </>
    );
};

export default InvoiceList;
