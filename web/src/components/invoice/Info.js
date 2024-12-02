import {ListGroup} from "react-bootstrap";
import AmountDisplay from "../common/AmountDisplay";
import {getNetwork} from "../../utils";
import LocalDate from "../common/LocalDate";
import React from "react";

function Info({invoice}) {
    return (
        <ListGroup variant="flush" className="mb-4">
            <ListGroup.Item>
                <strong>Invoice ID:</strong> {invoice.id}
            </ListGroup.Item>
            <ListGroup.Item>
                <strong>Amount:</strong> <AmountDisplay amount={invoice.amount} color={"text-success"}/>
            </ListGroup.Item>
            <ListGroup.Item>
                <strong>Networks:</strong>
                <div className="mt-2">
                    {invoice.networks.length > 0 ? (
                        <ListGroup variant="flush">
                            {invoice.networks.map((n) => (
                                <ListGroup.Item key={n} className="border-0 ps-3">
                                    – {getNetwork(n).name}
                                </ListGroup.Item>
                            ))}
                        </ListGroup>
                    ) : (
                        <span className="ps-3">No networks available</span>
                    )}
                </div>
            </ListGroup.Item>
            <ListGroup.Item>
                <strong>Seller:</strong> {invoice.seller}
            </ListGroup.Item>
            <ListGroup.Item>
                <strong>Created At:</strong>{' '}
                <LocalDate date={invoice.created_at}/>
            </ListGroup.Item>
            {invoice.paid_at && (
                <ListGroup.Item>
                    <strong>Paid At:</strong>{' '}
                    <LocalDate date={invoice.paid_at}/>
                </ListGroup.Item>
            )}
        </ListGroup>
    )
}

export default Info;