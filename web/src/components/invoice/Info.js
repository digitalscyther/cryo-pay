import {ListGroup} from "react-bootstrap";
import AmountDisplay from "../common/AmountDisplay";
import {getNetwork} from "../../utils";
import LocalDate from "../common/LocalDate";
import React from "react";
import NetworkIcon from "../NetworkIcon";

function Info({invoice}) {
    return (
        <ListGroup variant="flush" className="mb-4">
            <ListGroup.Item>
                <strong>Invoice ID:</strong> {invoice.id}
            </ListGroup.Item>
            {invoice.external_id && (
                <ListGroup.Item>
                    <strong>External ID:</strong> {invoice.external_id}
                </ListGroup.Item>
            )}
            <ListGroup.Item>
                <strong>Amount:</strong> <AmountDisplay amount={invoice.amount} color={"text-success"}/>
            </ListGroup.Item>
            <ListGroup.Item>
                <strong>Networks:</strong>
                <div className="mt-2 d-flex">
                    {invoice.networks.length > 0 ? (
                        <>
                            {invoice.networks.map(getNetwork).map((n) => n.name).sort().map((n) => (
                                <div key={n} className="m-3">
                                    <NetworkIcon size={40} networkName={n} />
                                </div>
                            ))}
                        </>
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