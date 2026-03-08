use sqlx::PgPool;
use sqlx::types::Uuid;
use super::User;

pub async fn get_user_by_id(db: &PgPool, id: &Uuid) -> Result<User, sqlx::Error> {
    sqlx::query_as!(
        User,
        r#"
        SELECT * FROM "users"
        WHERE id = $1
        "#,
        id
    )
        .fetch_one(db)
        .await
}

pub async fn get_or_create_user(db: &PgPool, firebase_user_id: &str, email: Option<String>) -> Result<User, sqlx::Error> {
    let existing_user = sqlx::query_as!(
        User,
        r#"
        SELECT * FROM "users"
        WHERE firebase_user_id = $1 AND email = $2
        "#,
        firebase_user_id,
        email
    )
        .fetch_optional(db)
        .await?;

    if let Some(user) = existing_user {
        return Ok(user);
    }

    let new_user = sqlx::query_as!(
        User,
        r#"
        INSERT INTO "users" (firebase_user_id, email)
        VALUES ($1, $2)
        ON CONFLICT (firebase_user_id)
        DO UPDATE SET email = EXCLUDED.email
        RETURNING *
        "#,
        firebase_user_id,
        email
    )
        .fetch_one(db)
        .await?;

    Ok(new_user)
}

pub async fn update_user(
    db: &PgPool,
    user_id: &Uuid,
    email_notification: Option<bool>,
    telegram_notification: Option<bool>,
) -> Result<User, sqlx::Error> {
    if let Some(email_notification) = email_notification {
        if let Some(telegram_notification) = telegram_notification {
            return sqlx::query_as!(
                User,
                r#"
                UPDATE "users"
                SET email_notification = $1, telegram_notification = $2
                WHERE id = $3
                RETURNING *
                "#,
                email_notification,
                telegram_notification,
                user_id,
            )
                .fetch_one(db)
                .await;
        }

        return sqlx::query_as!(
            User,
            r#"
            UPDATE "users"
            SET email_notification = $1
            WHERE id = $2
            RETURNING *
            "#,
            email_notification,
            user_id,
        )
            .fetch_one(db)
            .await;
    }

    if let Some(telegram_notification) = telegram_notification {
        return sqlx::query_as!(
            User,
            r#"
            UPDATE "users"
            SET telegram_notification = $1
            WHERE id = $2
            RETURNING *
            "#,
            telegram_notification,
            user_id,
        )
            .fetch_one(db)
            .await;
    }

    get_user_by_id(db, user_id).await
}

pub async fn set_user_telegram_chat_id(db: &PgPool, id: &Uuid, telegram_chat_id: Option<String>) -> Result<(), sqlx::Error> {
    sqlx::query_as!(
        User,
        r#"
        UPDATE "users"
        SET telegram_chat_id = $1
        WHERE id = $2
        "#,
        telegram_chat_id,
        id,
    )
        .execute(db)
        .await?;

    Ok(())
}
