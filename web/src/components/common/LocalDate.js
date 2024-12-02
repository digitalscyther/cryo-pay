import React from 'react';

const LocalDate = ({date}) => {
    const formatDateTime = (dateString) => {
        if (!dateString) return 'N/A';
        return new Date(dateString + "Z").toLocaleString();
    };

    return (
        <span>
            {formatDateTime(date)}
        </span>
    );
};

export default LocalDate;
