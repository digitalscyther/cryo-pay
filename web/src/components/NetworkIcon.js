import {getBlockchainIconPath} from "../utils";
import {Image, OverlayTrigger, Tooltip} from "react-bootstrap";
import React from "react";

const capitalizeWords = (str) => {
    return str
        .replace(/-/g, ' ')
        .replace(/\b\w/g, char => char.toUpperCase());
};


const BaseImage = ({size, networkName, cursor}) => {
    return (
        <div style={{height: `${size}px`, width: `${size}px`, cursor: `${cursor || 'help'}`}}>
            <Image
                src={getBlockchainIconPath(networkName)}
                alt={networkName}
                fluid
                style={{height: '100%', width: '100%', objectFit: 'contain'}}
            />
        </div>
    )
}

const NetworkIcon = ({size, networkName, noTooltip, cursor, helperSide= 'top'}) => {
    if (noTooltip) {
        return (
            <BaseImage size={size} networkName={networkName} cursor={cursor} />
        )
    }

    return (
        <OverlayTrigger
            placement={helperSide}
            overlay={<Tooltip>{capitalizeWords(networkName)}</Tooltip>}
        >
            <div><BaseImage size={size} networkName={networkName} cursor={cursor} /></div>
        </OverlayTrigger>
    )
}

export default NetworkIcon
