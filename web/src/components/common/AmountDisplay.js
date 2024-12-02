import React from 'react';
import PropTypes from 'prop-types';

const AmountDisplay = ({ amount, size, color }) => {
    let fontSize = size || 1.2;
    let textColor = color || "text-dark";

    return (
        <span className={textColor} style={{ fontSize: `${fontSize}em`}}>
            ${parseFloat(amount).toFixed(2)}
        </span>
    );
};

AmountDisplay.propTypes = {
    amount: PropTypes.oneOfType([PropTypes.string, PropTypes.number]).isRequired,
};

export default AmountDisplay;
