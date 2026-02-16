export const parseBackendDate = (dateInput: any): Date => {
  if (!dateInput) return new Date();

  // If it's an object with a timestamp property (sometimes happens with complex serialization)
  if (typeof dateInput === 'object' && dateInput !== null && 'timestamp' in dateInput) {
    return parseBackendDate(dateInput.timestamp);
  }

  // Handle numeric timestamps (in seconds or milliseconds)
  if (typeof dateInput === 'number' || (!isNaN(Number(dateInput)) && !String(dateInput).includes('-') && !String(dateInput).includes(':'))) {
    const num = Number(dateInput);
    // If it's a timestamp in seconds (e.g. 1.7e9), convert to ms
    if (num > 1e9 && num < 1e12) {
      return new Date(num * 1000);
    }
    return new Date(num);
  }

  // Try standard parsing first
  let date = new Date(dateInput);
  if (!isNaN(date.getTime())) {
    return date;
  }

  // Handle Rust 'time' crate default format: "YYYY-MM-DD HH:MM:SS.N +HH:MM:SS"
  // Transform to ISO: "YYYY-MM-DDTHH:MM:SS.N+HH:MM"
  
  if (typeof dateInput === 'string') {
     // 1. Replace date-time separator space with T
     // Regex: Match YYYY-MM-DD space HH:MM
     let isoStr = dateInput.replace(/^(\d{4}-\d{2}-\d{2})\s(\d{2}:\d{2})/, '$1T$2');
     
     // 2. Remove space before timezone offset if present
     isoStr = isoStr.replace(/\s+([+-]\d)/, '$1');

     // 3. Fix offset: +00:00:00 -> +00:00
     // Remove the seconds part from the offset if present
     isoStr = isoStr.replace(/([+-]\d{2}:\d{2}):\d{2}$/, '$1');
     
     date = new Date(isoStr);
     if (!isNaN(date.getTime())) {
       return date;
     }
  }

  // Fallback to current date to avoid breaking the UI
  console.warn(`Unable to parse date: ${dateInput}`);
  return new Date();
};
