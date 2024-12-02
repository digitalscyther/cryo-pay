import React, { useEffect, useState } from "react";
import { Button, OverlayTrigger, Tooltip } from "react-bootstrap";

function MetaMaskButton({ onPress }) {
    const [isMetaMaskAvailable, setIsMetaMaskAvailable] = useState(false);

    useEffect(() => {
        setIsMetaMaskAvailable(!!window.ethereum);
    }, []);

    return (
        <div>
            {isMetaMaskAvailable ? (
                <Button
                    variant="outline-primary" className="ms-2"
                    onClick={onPress}
                >
                    Use MetaMask
                </Button>
            ) : (
                <OverlayTrigger
                    placement="top"
                    overlay={
                        <Tooltip>
                            MetaMask is not available. Please install it to use this feature.
                        </Tooltip>
                    }
                >
                    <span>
                        <Button variant="outline-secondary" className="ms-2" disabled>
                            Use MetaMask
                        </Button>
                    </span>
                </OverlayTrigger>
            )}
        </div>
    );
}

export default MetaMaskButton;
