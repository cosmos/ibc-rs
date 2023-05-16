pub fn schedule_upgrade<Ctx>(ctx: &mut Ctx, plan: Plan) -> Result<(), ContextError>
where
    Ctx: UpgradeExecutionContext,
{
    let plan = ctx.get_upgrade_plan()?;
    let upgraded_client_state_path =
        UpgradeClientPath::UpgradedClientState(plan.height.revision_height());

    ctx.upgraded_client_state(&upgraded_client_state_path)?;

    if ctx.host_height()? == plan.height.decrement()? {
        let upgraded_consensus_state = ctx.host_consensus_state(&ctx.host_height()?)?;
        let upgraded_cons_state_path =
            UpgradeClientPath::UpgradedClientConsensusState(plan.height.revision_height());

        ctx.store_upgraded_consensus_state(upgraded_cons_state_path, upgraded_consensus_state)?;

        ctx.emit_upgrade_chain_event(plan.height)?;
    }
    Ok(())
}

pub fn begin_blocker<Ctx>(ctx: &mut Ctx) -> Result<Vec<Event>, ContextError>
where
    Ctx: UpgradeExecutionContext,
{
    let plan = ctx.get_upgrade_plan()?;
    let upgraded_client_state_path =
        UpgradeClientPath::UpgradedClientState(plan.height.revision_height());

    ctx.upgraded_client_state(&upgraded_client_state_path)?;

    if ctx.host_height()? == plan.height.decrement()? {
        let upgraded_consensus_state = ctx.host_consensus_state(&ctx.host_height()?)?;
        let upgraded_cons_state_path =
            UpgradeClientPath::UpgradedClientConsensusState(plan.height.revision_height());

        ctx.store_upgraded_consensus_state(upgraded_cons_state_path, upgraded_consensus_state)?;

        ctx.emit_upgrade_chain_event(plan.height)?;
    }
    Ok(())
}
