import { createContext } from 'react';

import { UUID } from '../../generated/UUID';

const TriggerIDsInSelectedTriggerTagContext = createContext<{
  tagID: UUID;
  triggerIDs: Set<UUID>;
} | null>(null);

export default TriggerIDsInSelectedTriggerTagContext;
