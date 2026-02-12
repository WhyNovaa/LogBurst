export const parseBackendDate = (dateStr: string): Date => {
  // Try standard parsing first
  let date = new Date(dateStr);
  if (!isNaN(date.getTime())) {
    return date;
  }

  // Handle Rust 'time' crate default format: "YYYY-MM-DD HH:MM:SS.N +HH:MM:SS"
  // Transform to ISO: "YYYY-MM-DDTHH:MM:SS.N+HH:MM"
  
  if (typeof dateStr === 'string') {
     // 1. Replace date-time separator space with T
     // Regex: Match YYYY-MM-DD space HH:MM
     let isoStr = dateStr.replace(/^(\d{4}-\d{2}-\d{2})\s(\d{2}:\d{2})/, '$1T$2');
     
     // 2. Fix offset: +00:00:00 -> +00:00
     // Remove the seconds part from the offset if present
     isoStr = isoStr.replace(/([+-]\d{2}:\d{2}):\d{2}$/, '$1');
     
     date = new Date(isoStr);
     if (!isNaN(date.getTime())) {
       return date;
     }
  }

  // Fallback to current date or throw to be caught by caller
  throw new Error(`Invalid date format: ${dateStr}`);
};
