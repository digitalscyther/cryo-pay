import React, { useEffect, useState } from 'react';
import { Container, Form, Button, Alert, Spinner } from 'react-bootstrap';
import axios from 'axios';
import ApiKeys from "./ApiKeys";
import CallbackUrls from "./CallbackUrls";
import { apiUrl } from '../../utils';

function Account() {
    const [settings, setSettings] = useState({
        // email_notification: false,   // TODO notification_turned_off
        telegram_notification: false,
    });
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState(null);
    const [attachTelegramPath, setAttachTelegramPath] = useState(null); // Tracks Telegram bot attachment URL

    // Fetch current settings on mount
    useEffect(() => {
        const fetchSettings = async () => {
            try {
                const response = await axios.get(apiUrl('/user'), { withCredentials: true });
                setSettings(response.data);
                setAttachTelegramPath(response.data.attach_telegram_path);
            } catch (err) {
                setError("Error loading settings.");
            } finally {
                setLoading(false);
            }
        };
        fetchSettings();
    }, []);

    // Automatically update settings when a toggle changes
    const updateSetting = async (updatedSettings) => {
        setError(null);

        try {
            const response = await axios.patch(apiUrl('/user'), updatedSettings, { withCredentials: true });
            setSettings(response.data);
            setAttachTelegramPath(response.data.attach_telegram_path);
        } catch (err) {
            setError("Failed to update settings.");
        }
    };

    const handleChange = (e) => {
        const { name, checked } = e.target;
        const newSettings = { ...settings, [name]: checked };
        setSettings(newSettings);
        updateSetting(newSettings);
    };

    const handleActivateBot = () => {
        window.open(apiUrl(attachTelegramPath), '_blank');
    };

    if (loading) return <div className="text-center"><Spinner animation="border" /></div>;

    return (
        <Container>
            <>
                <h3>Notification Settings</h3>

                <Form className="my-4 mx-3">
                    <div className="d-flex">
                        <Form.Check
                            type="switch"
                            id="email-notification"
                            label="Email"
                            name="email_notification"
                            // checked={settings.email_notification}    // TODO notification_turned_off
                            checked={false}
                            onChange={handleChange}
                            disabled    // TODO notification_turned_off
                        />
                        <div className="ms-5 text-warning">Available only by subscription</div>
                    </div>
                    <Form.Check
                        type="switch"
                        id="telegram-notification"
                        label="Telegram"
                        name="telegram_notification"
                        checked={settings.telegram_notification}
                        onChange={handleChange}
                    />
                </Form>

                {error && <Alert variant="danger">{error}</Alert>}

                {/* Telegram Bot Warning */}
                {attachTelegramPath && (
                    <Alert variant="warning" className="d-flex justify-content-between align-items-center">
                        <span>Activate the bot to receive Telegram notifications.</span>
                        <Button onClick={handleActivateBot} variant="outline-primary" size="sm">Activate Bot</Button>
                    </Alert>
                )}
            </>
            <hr/>
            {/* Callback Urls Section */}
            <CallbackUrls/>
            {/* API Keys Section */}
            <ApiKeys/>
        </Container>
    );
}

export default Account;
