import React, { useState, useEffect } from 'react';
import { Alert, Button, ButtonGroup, Card, Col, Row, Spinner, Table } from 'react-bootstrap';
import axios from 'axios';
import { apiUrl } from '../../utils';

function Analytics() {
    const [data, setData] = useState(null);
    const [days, setDays] = useState(30);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState(null);

    useEffect(() => {
        setLoading(true);
        setError(null);
        axios
            .get(apiUrl(`/user/analytics?days=${days}`), { withCredentials: true })
            .then((r) => {
                setData(r.data);
                setLoading(false);
            })
            .catch(() => {
                setError('Failed to load analytics');
                setLoading(false);
            });
    }, [days]);

    return (
        <div className="mt-4">
            <div className="d-flex align-items-center justify-content-between mb-3">
                <h4 className="mb-0">Analytics</h4>
                <ButtonGroup size="sm">
                    {[7, 30, 90].map((d) => (
                        <Button
                            key={d}
                            variant={days === d ? 'primary' : 'outline-primary'}
                            onClick={() => setDays(d)}
                        >
                            {d}d
                        </Button>
                    ))}
                </ButtonGroup>
            </div>

            {loading && (
                <div className="text-center my-4">
                    <Spinner animation="border" variant="primary" />
                </div>
            )}

            {error && (
                <Alert variant="danger">{error}</Alert>
            )}

            {!loading && !error && data && (
                <>
                    <Row className="g-3 mb-4">
                        <Col xs={6} md={3}>
                            <Card className="h-100 text-center">
                                <Card.Body>
                                    <Card.Title className="fs-2 mb-1">
                                        {data.summary.total_invoices}
                                    </Card.Title>
                                    <Card.Text className="text-muted small">Total Invoices</Card.Text>
                                </Card.Body>
                            </Card>
                        </Col>
                        <Col xs={6} md={3}>
                            <Card className="h-100 text-center">
                                <Card.Body>
                                    <Card.Title className="fs-2 mb-1">
                                        {data.summary.paid_invoices}
                                    </Card.Title>
                                    <Card.Text className="text-muted small">Paid Invoices</Card.Text>
                                </Card.Body>
                            </Card>
                        </Col>
                        <Col xs={6} md={3}>
                            <Card className="h-100 text-center">
                                <Card.Body>
                                    <Card.Title className="fs-2 mb-1">
                                        {Number(data.summary.total_amount).toFixed(2)}
                                    </Card.Title>
                                    <Card.Text className="text-muted small">Total USDT</Card.Text>
                                </Card.Body>
                            </Card>
                        </Col>
                        <Col xs={6} md={3}>
                            <Card className="h-100 text-center">
                                <Card.Body>
                                    <Card.Title className="fs-2 mb-1">
                                        {Number(data.summary.paid_amount).toFixed(2)}
                                    </Card.Title>
                                    <Card.Text className="text-muted small">Paid USDT</Card.Text>
                                </Card.Body>
                            </Card>
                        </Col>
                    </Row>

                    {data.by_day.length === 0 ? (
                        <Alert variant="info">No invoices in the last {data.period_days} days.</Alert>
                    ) : (
                        <Table striped bordered hover responsive size="sm">
                            <thead>
                                <tr>
                                    <th>Date</th>
                                    <th className="text-end">Total</th>
                                    <th className="text-end">Paid</th>
                                    <th className="text-end">Amount (USDT)</th>
                                    <th className="text-end">Paid (USDT)</th>
                                </tr>
                            </thead>
                            <tbody>
                                {data.by_day.map((row) => (
                                    <tr key={row.period}>
                                        <td>{new Date(row.period).toLocaleDateString()}</td>
                                        <td className="text-end">{row.total_invoices}</td>
                                        <td className="text-end">{row.paid_invoices}</td>
                                        <td className="text-end">{Number(row.total_amount).toFixed(2)}</td>
                                        <td className="text-end">{Number(row.paid_amount).toFixed(2)}</td>
                                    </tr>
                                ))}
                            </tbody>
                        </Table>
                    )}
                </>
            )}
        </div>
    );
}

export default Analytics;
