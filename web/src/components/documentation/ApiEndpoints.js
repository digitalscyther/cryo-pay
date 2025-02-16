import {Alert, Table, Row, Col} from "react-bootstrap";
import {apiUrl, exampleEthereumAddressHere, getFullUrl, NETWORKS} from "../../utils";

function getFullApiUrl(path) {
    path = apiUrl(path);

    if (path.startsWith("http")) {
        return path;
    }

    return getFullUrl(path);
}

class Optional {
    constructor(value, type = null) {
        this.value = value;
        this.type = type;
    }

    as_str() {
        let type = this.type || typeof this.value;
        return `Optional(${type})`
    }
}

const ENDPOINTS = [
    {
        "path": "/payment/invoice",
        "methods": [
            {
                "info": {
                    "name": "List Invoices",
                    "description": "Retrieve a list of invoices with optional pagination parameters.",
                },
                "request": {
                    "method": "get",
                    "query_params": [
                        {
                            "name": "limit",
                            "description": "The maximum number of invoices to return (default: 10).",
                            "value": new Optional(10)
                        },
                        {
                            "name": "offset",
                            "description": "The number of invoices to skip before starting to return results (default: 0).",
                            "value": new Optional(0)
                        },
                        {
                            "name": "user_id",
                            "description": "Filter invoices based on ownership. Use 'all' for all invoices or 'my' for only the authenticated user’s invoices (default: 'all').",
                            "value": new Optional("my")
                        }
                    ]
                },
                "responses": [
                    {
                        "status": 200,
                        "json": [
                            {
                                "id": "90ac15d3-58f8-4bf8-a899-61c64ba8693e",
                                "created_at": "2025-02-14T23:05:16.272317",
                                "amount": "1",
                                "seller": exampleEthereumAddressHere,
                                "paid_at": "2025-02-14T23:05:38",
                                "networks": [
                                    11155420,
                                    10,
                                    42161
                                ],
                                "external_id": "Donation of 1"
                            },
                            {
                                "id": "fd0c6d66-b06d-49a1-87f7-8a3d8234220c",
                                "created_at": "2025-02-14T20:19:54.078568",
                                "amount": "111",
                                "seller": exampleEthereumAddressHere,
                                "paid_at": null,
                                "networks": [
                                    11155420
                                ],
                                "external_id": null
                            }
                        ],
                    }
                ]
            },
            {
                "info": {
                    "name": "Create Invoice",
                    "description": "Create a new invoice for a payment request.",
                },
                "request": {
                    "method": "post",
                    "json": [
                        {
                            "name": "amount",
                            "description": "The total amount of the invoice in decimal format (e.g., '150.00').",
                            "value": "150.00"
                        },
                        {
                            "name": "seller",
                            "description": "The Ethereum address of the seller receiving the payment.",
                            "value": exampleEthereumAddressHere
                        },
                        {
                            "name": "networks",
                            "description": "A list of supported network IDs (e.g., 10 for Optimism, 42161 for Arbitrum).",
                            "value": new Array([10, 42161])
                        },
                        {
                            "name": "external_id",
                            "description": "An optional custom identifier for tracking the invoice (experimental feature).",
                            "value": new Optional("custom id (experimental)")
                        }
                    ]
                },
                "responses": [
                    {
                        "status": 200,
                        "json": {
                            "id": "b92d6367-6bf1-49b8-8180-d7fb79d7c75b",
                            "created_at": "2025-02-15T17:18:54.976321",
                            "amount": "9.12",
                            "seller": exampleEthereumAddressHere,
                            "paid_at": null,
                            "networks": [
                                10,
                                42161
                            ],
                            "external_id": "Foo"
                        }
                    },
                    {"status": 400},
                    {
                        "status": 429,
                        "json": {"error": "too_many_requests"}
                    },
                ]
            },
        ]
    },
    {
        "path": "/payment/invoice/:invoice_id",
        "methods": [
            {
                "info": {
                    "name": "Get Invoice",
                    "description": "Retrieve detailed information about a specific invoice by its ID.",
                },
                "request": {
                    "method": "get"
                },
                "responses": [
                    {
                        "status": 200,
                        "json": {
                            "id": "b92d6367-6bf1-49b8-8180-d7fb79d7c75b",
                            "created_at": "2025-02-15T17:18:54.976321",
                            "amount": "9.12",
                            "seller": exampleEthereumAddressHere,
                            "paid_at": null,
                            "networks": [
                                10,
                                42161
                            ],
                            "external_id": "Foo"
                        }
                    },
                    {
                        "status": 404,
                        "json": {"message": "not_found"}
                    }
                ]
            },
            {
                "info": {
                    "name": "Delete Invoice",
                    "description": "Delete an existing invoice by its ID.",
                },
                "request": {
                    "method": "delete"
                },
                "responses": [
                    {"status": 204},
                    {
                        "status": 404,
                        "json": {"message": "not_found"}
                    }
                ]
            }
        ],
    },
];

const Field = ({obj}) => {
    return (
        <p className="ms-3 my-0"><b>{obj.name}</b> - {
            obj.value instanceof Optional ? (
                <><code>{"<"}{obj.value.as_str()}{">"}</code> - example: <code>{obj.value.value}</code></>
            ) : (<><code>{"<"}{typeof obj.value}{">"}</code> - example: <code>{obj.value}</code></>)
        } - {obj.description}</p>
    )
}

const Endpoints = () => {
    return (
        <>
            <h6>📜 Available Invoice Endpoints</h6>
            <Table bordered hover>
                <thead>
                <tr>
                    <th>Info</th>
                    <th>Request</th>
                    <th>Response</th>
                </tr>
                </thead>
                <tbody>
                {ENDPOINTS.map((ep) => ep.methods.map((m) => (
                    <tr>
                        <td>
                            <Row className="d-flex flex-column">
                                <Col><b>{m.info.name}</b></Col>
                                <Col>{m.info.description}</Col>
                            </Row>
                        </td>
                        <td>
                            <Row className="d-flex flex-column">
                                <Col>{m.request.method.toUpperCase()}</Col>
                                <Col><code>{getFullApiUrl(ep.path)}</code></Col>
                                {m.request.query_params && (
                                    <Col>
                                        Query Params
                                        {m.request.query_params.map((field) => (<Field obj={field}/>))}
                                    </Col>
                                )}
                                {m.request.json && (
                                    <Col>
                                        Json
                                        {m.request.json.map((field) => (<Field obj={field}/>))}
                                    </Col>
                                )}
                            </Row>
                        </td>
                        <td>
                            {m.responses.map((r) => (
                                <Row className="d-flex flex-column">
                                    <Col>Status Code: <code>{r.status}</code></Col>
                                    {r.json && (
                                        <Col>JSON: <pre>{JSON.stringify(r.json, null, 2)}</pre></Col>
                                    )}
                                </Row>
                            ))}
                        </td>
                    </tr>
                )))}

                </tbody>
            </Table>
        </>
    )
}

function ApiEndpoints() {
    let url_api_invoices = getFullApiUrl("/payment/invoice");
    let invoice_id = "3d9de564-c1b9-4e81-b98d-58160d163002";
    let invoice_url = getFullUrl("/invoices/:invoice_id");
    let invoice_url_example = getFullUrl(`/invoices/${invoice_id}`);
    let invoice_url_example_no_nav_bar = getFullUrl(`/invoices/${invoice_id}?nnb=1`);
    let callback_url = "https://foo.bar/baz";
    let invoice_url_with_callback_url = getFullUrl(`/invoices/${invoice_id}?nnb=1&callback_url=${callback_url}`);
    let redirect_url = `${callback_url}?invoice_id=${invoice_id}&status=SUCCESS`;

    return (<>
        <Alert variant="primary">
            <h5 className="mb-3">🚀 API Endpoints</h5>
            <p>
                Our API currently supports <strong>invoice management</strong>{" "}
                functionalities. Below are the details of how to integrate and use the
                API securely and effectively.
            </p>
        </Alert>

        <h6>🔒 Authentication</h6>
        <p>
            Access to the API requires an <strong>API key</strong>. Generate your
            API key in the <em>Settings</em> section of your account. Include it in
            the request headers like this:
        </p>
        <pre>{`Authorization: Bearer YOUR_API_KEY`}</pre>
        <p>
            <strong>Note:</strong> API keys are available only to logged-in users.
        </p>

        <Endpoints/>

        <h6>📜 Request Payload for Creating an Invoice</h6>
        <p>Here’s how the payload for creating a new invoice looks:</p>
        <pre>
        {`POST ${url_api_invoices} HTTP/1.1
  Authorization: Bearer YOUR_API_KEY
  Content-Type: application/json
  
  {
      "amount": "150.00",
      "seller": "${exampleEthereumAddressHere}",
      "networks": [10, 42161]
  }`}
      </pre>
        <Alert variant="info">
            <strong>⚠️ Seller:</strong> This should be the Ethereum-based address
            (e.g., <code>0x...</code>) where funds will be accepted.
        </Alert>

        {/* New Section: Accessing and Sharing Invoices */}
        <h6>🔗 Accessing and Sharing Invoices</h6>
        <p>
            Once an invoice is created, it becomes available at a unique, shareable URL.
            The format of the URL is:
        </p>
        <p><code>{invoice_url}</code></p>
        <p>
            For example:
        </p>
        <p><code>{invoice_url_example}</code></p>
        <p>
            You can share this URL directly with your clients for payment.
        </p>
        <p><code>{invoice_url_example_no_nav_bar}</code></p>
        <p>
            To hide the navigation bar for the client (recommended) — ensuring they focus solely on the payment process
            — add the query parameter <code>nnb=1</code> to the URL.
        </p>

        {/* New Section: Callback URL for Post-Payment Redirection */}
        <h6>🔄 Post-Payment Redirection with Callback URL</h6>
        <p>
            You can specify a <code>callback_url</code> as a query parameter when sharing the invoice URL.
            After a successful payment, the user will be automatically redirected to this URL. This is useful for
            integrating the payment process with your application's workflow.
        </p>
        <p>
            Example of an invoice URL with a <code>callback_url</code>:
        </p>
        <p><code>{invoice_url_with_callback_url}</code></p>
        <p>
            In this example, after the payment is completed, the user will be redirected
            to <code>{redirect_url}</code>.
            You can use this to update your system about the payment status, display a custom message to the user,
            or
            trigger other actions in your application.
            Make sure that the provided <code>callback_url</code> is URL-encoded.
        </p>

        <h6>🌐 Supported Networks</h6>
        <Table striped bordered>
            <thead>
            <tr>
                <th>Name</th>
                <th>Network ID</th>
            </tr>
            </thead>
            <tbody>
            {Object.values(NETWORKS).map((network) => (<tr key={network.id}>
                <td>{network.name}</td>
                <td>{network.id}</td>
            </tr>))}
            </tbody>
        </Table>
    </>);
}

export default ApiEndpoints;
