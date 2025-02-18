import React, {useState} from "react";
import {Image, Modal} from "react-bootstrap";

const ThumbnailWithZoom = ({ src, altText, thumbnailWidthSize}) => {
  const [show, setShow] = useState(false);

  return (
    <>
      {/* Small SVG */}
      <img
        src={src}
        alt={altText}
        style={{ width: `${thumbnailWidthSize}`, cursor: 'zoom-in' }}
        onClick={() => setShow(true)}
      />

      {/* Modal with Full-Size SVG */}
      <Modal show={show} onHide={() => setShow(false)} centered>
        <Modal.Header closeButton>
        </Modal.Header>
        <Modal.Body className="d-flex justify-content-center">
          <Image
            src={src}
            alt={altText}
            fluid
            onClick={() => setShow(false)}
            style={{ cursor: 'zoom-out' }}
          />
        </Modal.Body>
      </Modal>
    </>
  );
}

export default ThumbnailWithZoom;
