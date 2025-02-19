import React, {useEffect, useState} from "react";
import {useSearchParams} from "react-router-dom";
import {Button, Image, Modal} from "react-bootstrap";

const ThumbnailWithZoom = ({src, altText, thumbnailWidthSize, uniqueId}) => {
    const [searchParams, setSearchParams] = useSearchParams();
    const [show, setShow] = useState(false);

    useEffect(() => {
        setShow(searchParams.get("zoom") === uniqueId);
    }, [searchParams, uniqueId]);

    const handleShow = () => {
        const newParams = new URLSearchParams(searchParams);
        newParams.set("zoom", uniqueId);
        setSearchParams(newParams);
    };

    const handleClose = () => {
        const newParams = new URLSearchParams(searchParams);
        newParams.delete("zoom");
        setSearchParams(newParams);
    };

    return (
        <>
            {/* Thumbnail */}
            <img
                src={src}
                alt={altText}
                style={{width: `${thumbnailWidthSize}`, cursor: "zoom-in"}}
                onClick={handleShow}
            />

            {/* Modal */}
            <Modal show={show} onHide={handleClose} centered>
                <Modal.Header closeButton/>
                <Modal.Body className="d-flex justify-content-center">
                    <Image
                        src={src}
                        alt={altText}
                        fluid
                        onClick={handleClose}
                        style={{cursor: "zoom-out"}}
                    />
                </Modal.Body>
                <Modal.Footer>
                    <Button variant="secondary" onClick={handleClose}>
                        Close
                    </Button>
                </Modal.Footer>
            </Modal>
        </>
    );
};

export default ThumbnailWithZoom;
