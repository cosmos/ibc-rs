use crate::core::{ics02_client::height::Height, ContextError, ValidationContext};

use super::{connection::ConnectionEnd, error::ConnectionError};

pub fn verify_conn_delay_passed<Ctx>(
    ctx: &Ctx,
    packet_proof_height: Height,
    connection_end: &ConnectionEnd,
) -> Result<(), ContextError>
where
    Ctx: ValidationContext,
{
    // Fetch the current host chain time and height.
    let current_host_time = ctx.host_timestamp()?;
    let current_host_height = ctx.host_height()?;

    // Fetch the latest time and height that the counterparty client was updated on the host chain.
    let client_id = connection_end.client_id();
    let last_client_update_time = ctx.client_update_time(client_id, &packet_proof_height)?;
    let last_client_update_height = ctx.client_update_height(client_id, &packet_proof_height)?;

    // Fetch the connection delay time and height periods.
    let conn_delay_time_period = connection_end.delay_period();
    let conn_delay_height_period = ctx.block_delay(&conn_delay_time_period);

    // Verify that the current host chain time is later than the last client update time
    let earliest_valid_time = (last_client_update_time + conn_delay_time_period)
        .map_err(ConnectionError::TimestampOverflow)?;
    if current_host_time < earliest_valid_time {
        return Err(ConnectionError::NotEnoughTimeElapsed {
            current_host_time,
            earliest_valid_time,
        })
        .map_err(ContextError::ConnectionError);
    }

    // Verify that the current host chain height is later than the last client update height
    let earliest_valid_height = last_client_update_height.add(conn_delay_height_period);
    if current_host_height < earliest_valid_height {
        return Err(ConnectionError::NotEnoughBlocksElapsed {
            current_host_height,
            earliest_valid_height,
        })
        .map_err(ContextError::ConnectionError);
    };

    Ok(())
}
