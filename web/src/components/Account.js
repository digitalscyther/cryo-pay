import React, { useEffect, useState } from 'react';
import { Container, Form, Button, Alert, Spinner } from 'react-bootstrap';
import axios from 'axios';
import { apiUrl } from '../utils';

function Account() {
    const [settings, setSettings] = useState({
        email_notification: false,
        telegram_notification: false,
    });
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState(null);
    const [withChatId, setWithChatId] = useState(true); // Tracks Telegram bot connection

    // Fetch current settings on mount
    useEffect(() => {
        const fetchSettings = async () => {
            try {
                const response = await axios.get(apiUrl('/user'), { withCredentials: true });
                setSettings(response.data);
                setWithChatId(response.data.with_chat_id); // Update withChatId status
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
            setWithChatId(response.data.with_chat_id); // Check if user has connected bot
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
        // Redirect user to Telegram bot activation or show relevant instructions.
        window.open('https://t.me/YourBotUsername', '_blank'); // Adjust link to your bot
    };

    if (loading) return <div className="text-center"><Spinner animation="border" /></div>;

    return (
        <Container>
            <h3>Notification Settings</h3>

            {error && <Alert variant="danger">{error}</Alert>}

            {/* Telegram Bot Warning */}
            {settings.telegram_notification && !withChatId && (
                <Alert variant="warning" className="d-flex justify-content-between align-items-center">
                    <span>Activate the bot to receive Telegram notifications.</span>
                    <Button onClick={handleActivateBot} variant="outline-primary" size="sm">Activate Bot</Button>
                </Alert>
            )}

            <Form>
                <Form.Check
                    type="switch"
                    id="email-notification"
                    label="Email"
                    name="email_notification"
                    checked={settings.email_notification}
                    onChange={handleChange}
                />
                <Form.Check
                    type="switch"
                    id="telegram-notification"
                    label="Telegram"
                    name="telegram_notification"
                    checked={settings.telegram_notification}
                    onChange={handleChange}
                />
            </Form>
        </Container>
    );
}

export default Account;
