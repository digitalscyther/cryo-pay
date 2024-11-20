import React from 'react';

const LocalDate = ({date}) => {
    const formatDateTime = (dateString) => {
        if (!dateString) return 'N/A';
        return new Date(dateString + "z").toLocaleString();
    };

    return (
        <div>
            {formatDateTime(date)}
        </div>
    );
};

export default LocalDate;
